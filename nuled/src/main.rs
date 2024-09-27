#![no_std]
#![no_main]

use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::prelude::{FromValueType, Peripherals};
use esp_idf_hal::spi::{Dma, SpiBusDriver, SpiDriver};
use esp_idf_hal::spi::config::Config;
use esp_idf_sys::{useconds_t, usleep};
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::{brightness, SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

pub mod intervals {
    use esp_idf_sys::useconds_t;

    pub const SECOND: useconds_t = 1_000_000;
    pub const HALF_SECOND: useconds_t = 500_000;
    pub const QUARTER_SECOND: useconds_t = 250_000;
    pub const SIXTH_SECOND: useconds_t = 166_667;
    pub const EIGTHT_SECOND: useconds_t = 125_000;
    pub const TWELWTH_SECOND: useconds_t = 83_334;
    pub const SIXTEENTH_SECOND: useconds_t = 61_250;

    pub const IMMEDIATELY: useconds_t = 0;
}

const LED_COUNT: usize = 60;

#[no_mangle]
unsafe fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();
    let spi2 = peripherals.spi2.into_ref();

    let driver = SpiDriver::new_without_sclk(
        spi2,
        peripherals.pins.gpio8,
        Option::<AnyIOPin>::None,
        &esp_idf_hal::spi::config::DriverConfig::new().dma(Dma::Auto(512)),
    ).unwrap();

    let spi_bus = SpiBusDriver::new(
        driver,
        &Config::new().baudrate(3_200.kHz().into()),
    ).unwrap();

    let mut ws = Ws2812::new(spi_bus);

    // Boot sequence
    {
        led_blink(
            &mut ws,
            RGB8::new(0, 255, 0),
            RGB8::new(0, 0, 0),
            intervals::SECOND,
            intervals::TWELWTH_SECOND,
            1,
        );
    }

    loop {
        for hue in 0..=255 {
            led_blink(
                &mut ws,
                hsv2rgb(Hsv { hue, sat: 255, val: 120 }),
                hsv2rgb(Hsv { hue, sat: 255, val: 30 }),
                intervals::TWELWTH_SECOND,
                intervals::TWELWTH_SECOND,
                1,
            );
        }
    }
}

pub unsafe fn led_blink<'a>(
    ws: &mut Ws2812<SpiBusDriver<'a, SpiDriver<'a>>>,
    on_color: RGB8,
    off_color: RGB8,
    duty_cycle_on_usec: useconds_t,
    duty_cycle_off_usec: useconds_t,
    cycles: usize,
) {
    let on = [on_color; LED_COUNT];
    let off = [off_color; LED_COUNT];

    for _ in 0..cycles {
        ws.write(on).unwrap();
        usleep(duty_cycle_on_usec);
        ws.write(off).unwrap();
        usleep(duty_cycle_off_usec);
    }
}

pub fn rainbow_cycle<'a>(ws: &mut Ws2812<SpiBusDriver<'a, SpiDriver<'a>>>) {
    let mut data = [RGB8::default(); 1];
    #[allow(clippy::infinite_iter)]
    (0..=255).cycle().for_each(|hue| {
        unsafe {
            usleep(10_000);
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