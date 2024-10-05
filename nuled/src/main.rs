#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::clock::{ClockControl, Clocks};
use esp_hal::dma::{DmaPriority};
use esp_hal::gpio::{GpioPin, Io};
use esp_hal::peripherals::{Peripherals, SPI2};
use esp_hal::prelude::*;
use esp_hal::riscv::_export::critical_section;
use esp_hal::rng::Rng;
use esp_hal::spi::SpiMode;
use esp_hal::system::SystemControl;
use esp_hal::timer::timg::{TimerGroup};
use smart_leds::hsv::Hsv;
use smart_leds::SmartLedsWrite;
use static_cell::StaticCell;
use ws2812_spi::prerendered::Ws2812;

const WIFI_SSID: &'static str = env!("NULED_WIFI_SSID");
const WIFI_PASSWORD: &'static str = env!("NULED_WIFI_PASSWORD");
const LED_COUNT: usize = 60;

static CLOCKS: StaticCell<Clocks> = StaticCell::new();

#[embassy_executor::task]
async fn task() {
    let mut i = 0;
    loop {
        i = i + 1;
        log::info!("Task running [#{i}]...");
        embassy_time::Timer::after_secs(1).await;
    }
}

#[main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger(log::LevelFilter::Info);

    log::info!("NULED booting.");

    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let clocks: &'static mut Clocks = CLOCKS.init(clocks);

    let embassy_timer = TimerGroup::new(peripherals.TIMG0, clocks);

    log::info!("Initializing embassy...");
    esp_hal_embassy::init(clocks, embassy_timer.timer0);

    log::info!("Spawning bullshit task...");
    spawner.must_spawn(task());
    log::info!("Spawning WiFi task...");
    spawner.must_spawn(wifi_task(peripherals.TIMG1, peripherals.RNG, peripherals.RADIO_CLK, peripherals.WIFI, clocks));
    log::info!("Spawning LED task...");
    spawner.must_spawn(led_task(peripherals.SPI2, io.pins.gpio8, peripherals.DMA, clocks));

    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn wifi_task(
    wifi_timer: esp_hal::peripherals::TIMG1,
    rng: esp_hal::peripherals::RNG,
    radio_clk: esp_hal::peripherals::RADIO_CLK,
    wifi: esp_hal::peripherals::WIFI,
    clocks: &'static Clocks<'static>,
) {
    log::info!("Initializing WiFi configuration.");

    let wifi_timer = TimerGroup::new(wifi_timer, clocks);
    let wifi_init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Wifi,
        wifi_timer.timer0,
        Rng::new(rng),
        radio_clk,
        &clocks,
    );

    match wifi_init {
        Ok(_) => {}
        Err(err) => {
            log::info!("wifi setup error: {:?}", err);
        }
    }
    let wifi_init = wifi_init.unwrap();

    log::info!("Configuring WiFi for station mode.");

    let wifi_config = esp_wifi::wifi::ClientConfiguration {
        ssid: WIFI_SSID.parse().unwrap(),
        password: WIFI_PASSWORD.parse().unwrap(),
        auth_method: esp_wifi::wifi::AuthMethod::WPA2Personal,
        ..Default::default()
    };
    let (_wifi_device, mut wifi_controller) =
        esp_wifi::wifi::new_with_config::<esp_wifi::wifi::WifiStaDevice>(
            &wifi_init,
            wifi,
            wifi_config,
        ).unwrap();

    log::info!("Starting WiFi controller.");

    wifi_controller.start().await.unwrap();

    log::info!("Connecting WiFi...");

    match wifi_controller.connect().await {
        Ok(_) => {
            log::info!("WiFi connect success.");
        }
        Err(err) => {
            log::error!("WiFi connect error: {:?}", err);
        }
    }
}

#[embassy_executor::task]
async fn led_task(spi: SPI2, pin: GpioPin<8>, dma: esp_hal::peripherals::DMA, clocks: &'static Clocks<'static>) {
    log::info!("Setting up DMA buffers.");

    let (
        tx_buffer,
        tx_descriptors,
        rx_buffer,
        rx_descriptors
    ) = esp_hal::dma_buffers!(512);

    let tx_dma = esp_hal::dma::DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();
    let rx_dma = esp_hal::dma::DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();

    log::info!("Initializing SPI driver at 3.2MHz");

    let dma = esp_hal::dma::Dma::new(dma);
    let dma_channel_0 = dma.channel0.configure(true, DmaPriority::Priority9);
    let spi = esp_hal::spi::master::Spi::new(
        spi,
        3_200.kHz(),
        SpiMode::Mode0,
        &clocks,
    )
        .with_mosi(pin)
        .with_dma(dma_channel_0)
        ;

    log::info!("Initializing SPI DMA bus...");

    let spi_driver = esp_hal::spi::master::SpiDmaBus::new(spi, tx_dma, rx_dma);

    let mut led_buffer = [0_u8; (LED_COUNT * 12) + 40];
    let mut ws = Ws2812::new(spi_driver, &mut led_buffer);

    log::info!("WS2812 driver started on SPI2 and GPIO8.");

    embassy_time::Timer::after_millis(1).await;

    loop {
        let sat = 255;
        for hue in [0, 85, 170] {
            for val in 0..=255 {
                let color = smart_leds::hsv::hsv2rgb(Hsv { hue, sat, val });
                let data = [color; LED_COUNT];
                critical_section::with(|_| {
                    ws.write(data).unwrap();
                });
            }
            embassy_time::Timer::after_millis(50).await;
        }
    }
}