#![no_std]
#![no_main]

mod rust_mqtt;
mod effect;
mod color;
mod mqtt;
mod config;

use core::str::FromStr;
use crate::effect::{Effect, Params};
use crate::config::*;
use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::clock::{ClockControl, Clocks};
use esp_hal::dma::DmaPriority;
use esp_hal::gpio::{GpioPin, Io};
use esp_hal::peripherals::{Peripherals, SPI2};
use esp_hal::prelude::*;
use esp_hal::riscv::_export::critical_section;
use esp_hal::rng::Rng;
use esp_hal::spi::SpiMode;
use esp_hal::system::SystemControl;
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::current_millis;
use esp_wifi::wifi::WifiController;
use heapless::{spsc};
use smart_leds::SmartLedsWrite;
use static_cell::StaticCell;
use ws2812_spi::prerendered::Ws2812;

#[main]
async fn main(spawner: Spawner) {
    // Default to INFO level logging unless RUST_LOG=trace|debug|...
    let log_level = option_env!("NULED_LOG_LEVEL").unwrap_or_default();
    let log_level = log::LevelFilter::from_str(log_level).unwrap_or_else(|_| log::LevelFilter::Info);
    esp_println::logger::init_logger(log_level);

    info!("NULED booting.");

    static CLOCKS: StaticCell<Clocks> = StaticCell::new();
    static NETWORK_STACK: StaticCell<embassy_net::Stack<esp_wifi::wifi::WifiDevice<'_, esp_wifi::wifi::WifiStaDevice>>> = StaticCell::new();
    static NETWORK_STACK_MEMORY: StaticCell<embassy_net::StackResources<3>> = StaticCell::new();
    static COMMAND_QUEUE: StaticCell<spsc::Queue::<mqtt::EffectCommand, 16>> = StaticCell::new();

    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let clocks: &'static mut Clocks = CLOCKS.init(clocks);

    let embassy_timer = TimerGroup::new(peripherals.TIMG0, clocks);

    info!("Initializing embassy...");
    esp_hal_embassy::init(clocks, embassy_timer.timer0);

    debug!("Initializing WiFi configuration...");

    let wifi_timer = TimerGroup::new(peripherals.TIMG1, clocks);
    let wifi_init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Wifi,
        wifi_timer.timer0,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    ).unwrap();

    debug!("Configuring WiFi for station mode.");

    let (wifi_interface, wifi_controller) =
        esp_wifi::wifi::new_with_mode(
            &wifi_init,
            peripherals.WIFI,
            esp_wifi::wifi::WifiStaDevice,
        ).unwrap();

    let network_config = embassy_net::Config::dhcpv4(Default::default());

    let seed = 1234; // very random, very secure seed

    let stack_resources: &'static mut _ = NETWORK_STACK_MEMORY.init(embassy_net::StackResources::<3>::new());

    let network_stack: &'static mut _ = NETWORK_STACK.init(
        embassy_net::Stack::new(
            wifi_interface,
            network_config,
            stack_resources,
            seed,
        )
    );

    let command_queue: &'static mut _ = COMMAND_QUEUE.init(
        spsc::Queue::<mqtt::EffectCommand, 16>::new()
    );

    let (mut producer, consumer) = command_queue.split();

    let _ = producer.enqueue(mqtt::EffectCommand::ChangeEffect(mqtt::Effect::Rainbow));

    spawner.must_spawn(wifi_task(wifi_controller));
    spawner.must_spawn(net_task(network_stack));
    spawner.must_spawn(mqtt::mqtt_task(network_stack, producer));
    spawner.must_spawn(led_task(peripherals.SPI2, io.pins.gpio8, peripherals.DMA, clocks, consumer));

    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static embassy_net::Stack<esp_wifi::wifi::WifiDevice<'static, esp_wifi::wifi::WifiStaDevice>>) {
    stack.run().await
}

#[embassy_executor::task]
async fn wifi_task(
    mut wifi_controller: WifiController<'static>,
) {
    use esp_wifi::wifi::*;
    use embassy_time::Duration;
    use embassy_time::Timer;

    info!("WiFi task started.");

    loop {
        if let WifiState::StaConnected = get_wifi_state() {
            wifi_controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await
        }

        if !matches!(wifi_controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: WIFI_SSID.try_into().unwrap(),
                password: WIFI_PASSWORD.try_into().unwrap(),
                auth_method: AuthMethod::WPA2Personal,
                ..Default::default()
            });
            wifi_controller.set_configuration(&client_config).unwrap();
            info!("Starting WiFi controller...");
            wifi_controller.start().await.unwrap();
            info!("WiFi started.");
        }

        info!("WiFi connecting...");

        match wifi_controller.connect().await {
            Ok(_) => {
                info!("WiFi connect success.");
            }
            Err(err) => {
                let msg = match err {
                    WifiError::NotInitialized => "not initialized",
                    WifiError::InternalError(_) => "internal error",
                    WifiError::Disconnected => "disconnected",
                    WifiError::UnknownWifiMode => "unknown wifi mode",
                };
                error!("WiFi connect error: {}", msg);
                Timer::after(Duration::from_millis(5000)).await;
            }
        }
    }
}

/// The LED task is responsible for displaying data on the LED strip in a timely manner.
#[embassy_executor::task]
async fn led_task(
    spi: SPI2,
    pin: GpioPin<8>,
    dma: esp_hal::peripherals::DMA,
    clocks: &'static Clocks<'static>,
    mut queue: spsc::Consumer<'static, mqtt::EffectCommand, 16>,
) {
    info!("LED task started.");
    info!("Setting up DMA buffers.");

    let (
        tx_buffer,
        tx_descriptors,
        rx_buffer,
        rx_descriptors
    ) = esp_hal::dma_buffers!(512);

    let tx_dma = esp_hal::dma::DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();
    let rx_dma = esp_hal::dma::DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();

    //info!("Initializing SPI driver at 3.2MHz");
    info!("Initializing SPI driver");

    let dma = esp_hal::dma::Dma::new(dma);
    let dma_channel_0 = dma.channel0.configure(true, DmaPriority::Priority9);
    let spi = esp_hal::spi::master::Spi::new(
        spi,
        3_800.kHz(),
        SpiMode::Mode0,
        &clocks,
    )
        .with_mosi(pin)
        .with_dma(dma_channel_0)
        ;

    info!("Initializing SPI DMA bus...");

    let spi_driver = esp_hal::spi::master::SpiDmaBus::new(spi, tx_dma, rx_dma);

    let mut led_buffer = [0_u8; (LED_COUNT * 24) + 40];
    let mut ws = Ws2812::new_sk6812w(spi_driver, &mut led_buffer);

    info!("WS2812 driver started on SPI2 and GPIO8.");

    use static_box::Box;
    let mut mem = [0_u8; 4096];
    let mut effect: Box<dyn Effect<LED_COUNT>> = Box::new(&mut mem, effect::Polyrhythm::<LED_COUNT>::default());
    let mut state = Params::default();

    loop {
        let Some(command) = queue.dequeue() else {
            embassy_time::Timer::after_millis(1).await;
            continue;
        };

        match command {
            mqtt::EffectCommand::ChangeEffect(eff) => {
                drop(effect);
                effect = match eff {
                    mqtt::Effect::Solid => Box::new(&mut mem, effect::Solid::<LED_COUNT>::default()),
                    mqtt::Effect::Rainbow => Box::new(&mut mem, effect::Rainbow::<LED_COUNT>::default()),
                    mqtt::Effect::Gradient => Box::new(&mut mem, effect::Gradient::<LED_COUNT>::default()),
                    mqtt::Effect::Polyrhythm => Box::new(&mut mem, effect::Polyrhythm::<LED_COUNT>::default()),
                };
                effect.configure(state.clone());
            }
            mqtt::EffectCommand::ConfigureParams(new_state) => {
                state = new_state;
                effect.configure(state.clone());
            }
        }

        let mut last_effect_millis = current_millis();

        // Run the current effect until it is exhausted, or the user has requested a new effect.
        while let Some(strip) = effect.next() {
            /// Maximum amount of time budget for one frame of animation.
            /// 42ms corresponds to just below 24 frames per second, which is sufficient
            /// for the eye to not notice individual frames, while at the same time
            /// allowing for effects that do expensive float computation.
            const EFFECT_RUNTIME_NOMINAL_MS: i64 = 42;

            // Maximum LED brightness regardless of other parameters.
            //const BRIGHTNESS: u8 = 127;

            // let data = smart_leds::brightness(
            //     smart_leds::gamma(data.iter().cloned()),
            //     BRIGHTNESS,
            // );
            //let rgb_values = strip.to_rgb8();
            let rgb_values = strip.to_rgbw();
            //let gamma_corrected = smart_leds::gamma(rgb_values.iter().cloned());

            let pre_write_ms = current_millis();
            critical_section::with(|_| {
                ws.write(rgb_values).expect("failed LED update")
            });
            debug!("LED critical section in {} ms", current_millis() - pre_write_ms);

            let effect_runtime = (current_millis() - last_effect_millis) as i64;
            last_effect_millis = current_millis();
            let sleep_time = EFFECT_RUNTIME_NOMINAL_MS - effect_runtime;

            if sleep_time.is_negative() {
                warn!("Effect iteration took too long, {effect_runtime} ms is above target of {EFFECT_RUNTIME_NOMINAL_MS} ms");
            }

            let sleep_time = sleep_time.clamp(1, EFFECT_RUNTIME_NOMINAL_MS) as u64;
            debug!("Effect iteration took {effect_runtime} ms, yielding task for {sleep_time} ms");

            embassy_time::Timer::after_micros(sleep_time).await;

            if let Some(_) = queue.peek() {
                break;
            }
        }
    }
}