#![no_std]
#![no_main]

use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::prelude::{FromValueType, Peripherals};
use esp_idf_hal::spi::config::Config;
use esp_idf_hal::spi::{Dma, SpiBusDriver, SpiDriver};
use esp_idf_sys::{usleep};
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::{RGB8};
use ws2812_spi::Ws2812;
use crate::intervals::SECOND;

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

    log::info!("NULED booted!");

    let peripherals = Peripherals::take().unwrap();
    let spi2 = peripherals.spi2.into_ref();

    /*
    let mut power_config = esp_idf_svc::sys::esp_pm_config_esp32c3_t {
        max_freq_mhz: 160,
        min_freq_mhz: 160,
        light_sleep_enable: false,
    };

    let result = esp!(unsafe { esp_idf_svc::sys::esp_pm_configure(&mut power_config as *mut _ as *const c_void) });
    result.unwrap();
     */

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

    // LED writer
    let mut ws = Ws2812::new(spi_bus);

    // Boot sequence finished
    led::fill(&mut ws, RGB8::new(0, 255, 0));
    usleep(SECOND);

    // Test program: blink through each color hue
    loop {
        for hue in 0..=255 {
            led::blink(
                &mut ws,
                hsv2rgb(Hsv { hue, sat: 255, val: 120 }),
                hsv2rgb(Hsv { hue, sat: 255, val: 30 }),
                intervals::QUARTER_SECOND,
                intervals::QUARTER_SECOND,
                1,
            );
        }
    }
}

pub mod led {
    use esp_idf_hal::spi::{SpiBusDriver, SpiDriver};
    use esp_idf_sys::{useconds_t, usleep};
    use smart_leds::{SmartLedsWrite, RGB8};
    use smart_leds::hsv::{hsv2rgb, Hsv};
    use ws2812_spi::Ws2812;
    use crate::{intervals, LED_COUNT};

    pub unsafe fn fill<'a>(
        ws: &mut Ws2812<SpiBusDriver<'a, SpiDriver<'a>>>,
        color: RGB8,
    ) {
        let on = [color; LED_COUNT];
        ws.write(on).unwrap();
    }

    pub unsafe fn blink<'a>(
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

    pub unsafe fn test_program<'a>(ws: &mut Ws2812<SpiBusDriver<'a, SpiDriver<'a>>>) {
        for hue in 0..=255 {
            blink(
                ws,
                hsv2rgb(Hsv { hue, sat: 255, val: 120 }),
                hsv2rgb(Hsv { hue, sat: 255, val: 30 }),
                intervals::QUARTER_SECOND,
                intervals::QUARTER_SECOND,
                1,
            );
        }
    }
}