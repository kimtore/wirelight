#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::clock::ClockControl;
use esp_hal::delay::Delay;
use esp_hal::dma::DmaPriority;
use esp_hal::gpio::Io;
use esp_hal::peripherals::Peripherals;
use esp_hal::prelude::*;
use esp_hal::riscv::_export::critical_section;
use esp_hal::rng::Rng;
use esp_hal::spi::SpiMode;
use esp_hal::system::SystemControl;
use esp_hal::timer::timg::TimerGroup;
use smart_leds::hsv::Hsv;
use smart_leds::SmartLedsWrite;
use ws2812_spi::Ws2812;

#[allow(dead_code)]
const WIFI_SSID: &'static str = env!("NULED_WIFI_SSID");
#[allow(dead_code)]
const WIFI_PASSWORD: &'static str = env!("NULED_WIFI_PASSWORD");
#[allow(dead_code)]
const LED_COUNT: usize = 60;

#[no_mangle]
fn main() {
    esp_println::logger::init_logger(log::LevelFilter::Info);

    log::info!("NULED booting.");

    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let dma = esp_hal::dma::Dma::new(peripherals.DMA);
    let dma_channel_0 = dma.channel0.configure(true, DmaPriority::Priority9);
    let delay = Delay::new(&clocks);

    let (
        tx_buffer,
        tx_descriptors,
        rx_buffer,
        rx_descriptors
    ) = esp_hal::dma_buffers!(512);

    log::info!("Setting up DMA buffers.");

    let tx_dma = esp_hal::dma::DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();
    let rx_dma = esp_hal::dma::DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();

    log::info!("Initializing SPI driver at 3.2MHz");

    let spi = esp_hal::spi::master::Spi::new(
        peripherals.SPI2,
        3_200.kHz(),
        SpiMode::Mode1,
        &clocks,
    )
        .with_mosi(io.pins.gpio8)
        .with_dma(dma_channel_0)
        ;

    log::info!("Initializing SPI DMA bus...");

    let spi_driver = esp_hal::spi::master::SpiDmaBus::new(spi, tx_dma, rx_dma);

    let mut ws = Ws2812::new(spi_driver);

    log::info!("WS2812 driver started on SPI2 and GPIO8.");

    let wifi_timer = TimerGroup::new(peripherals.TIMG1, &clocks).timer0;
    //let wifi_timer = PeriodicTimer::new(SystemTimer::new(peripherals.SYSTIMER).alarm0.into());
    let wifi_rng = Rng::new(peripherals.RNG);

    log::info!("Initializing WiFi configuration.");

    let wifi_init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Wifi,
        wifi_timer,
        wifi_rng,
        peripherals.RADIO_CLK,
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
            peripherals.WIFI,
            wifi_config,
        ).unwrap();

    log::info!("Starting WiFi controller.");

    wifi_controller.start().unwrap();

    log::info!("Connecting WiFi...");

    wifi_controller.connect().unwrap();

    log::info!("WiFi connect started.");

    loop {
        let sat = 255;
        for hue in [0, 85, 170] {
            for val in 0..=255 {
                let color = smart_leds::hsv::hsv2rgb(Hsv { hue, sat, val });
                let data = [color; LED_COUNT];
                critical_section::with(|_| {
                    ws.write(data).unwrap();
                });
                delay.delay_millis(50);
            }
        }
    }
}