#![no_std]
#![no_main]

use esp_idf_hal::prelude::{FromValueType, Peripherals};
use esp_idf_hal::spi::Dma;

#[no_mangle]
unsafe fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();
    let driver = esp_idf_hal::spi::SpiDriver::new_without_sclk(
        peripherals.spi2,
        peripherals.pins.gpio10,
        Option::<esp_idf_hal::gpio::AnyIOPin>::None,
        &esp_idf_hal::spi::config::DriverConfig::new().dma(Dma::Auto(512)),
    ).unwrap();

    let spi_bus = esp_idf_hal::spi::SpiBusDriver::new(
        driver,
        &esp_idf_hal::spi::SpiConfig::new().baudrate(3_200.kHz().into()),
    ).unwrap();

    let led_driver = ws2812_spi::Ws2812::new(spi_bus);
}