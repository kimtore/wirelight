#![no_std]
#![no_main]

use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::prelude::{FromValueType, Peripherals};
use esp_idf_hal::spi::{Dma, SpiBusDriver, SpiDriver};
use esp_idf_hal::spi::config::Config;
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::{brightness, SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

#[no_mangle]
unsafe fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();
    let driver = SpiDriver::new_without_sclk(
        peripherals.spi2,
        peripherals.pins.gpio8,
        Option::<AnyIOPin>::None,
        &esp_idf_hal::spi::config::DriverConfig::new().dma(Dma::Auto(512)),
    ).unwrap();

    led_task(driver);
}

#[derive(Debug)]
pub enum Error {}

pub fn led_task(driver: SpiDriver) {
    let bus = SpiBusDriver::new(driver, &Config::new().baudrate(3_200.kHz().into())).unwrap();
    let mut ws = Ws2812::new(bus);
    let mut data = [RGB8::default(); 1];
    #[allow(clippy::infinite_iter)]
    (0..=255).cycle().for_each(|hue| {
        unsafe {
            esp_idf_sys::usleep(10_000);
        }
        data[0] = hsv2rgb(Hsv {
            hue,
            sat: 255,
            val: 120,
        });
        let pixel = brightness(data.iter().cloned(), 30);
        ws.write(pixel).unwrap();
    });
}