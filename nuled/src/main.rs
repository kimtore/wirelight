#![no_std]
#![no_main]

use esp_idf_svc::hal::gpio::AnyIOPin;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::prelude::{FromValueType, Peripherals};
use esp_idf_svc::hal::spi::config::Config;
use esp_idf_svc::hal::spi::{self, Dma, SpiBusDriver, SpiDriver};
use esp_idf_svc::sys::usleep;
use esp_idf_svc::{sys, wifi};
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::RGB8;
use ws2812_spi::Ws2812;

pub mod intervals {
    use esp_idf_svc::sys::useconds_t;

    pub const SECOND: useconds_t = 1_000_000;
    pub const HALF_SECOND: useconds_t = 500_000;
    pub const QUARTER_SECOND: useconds_t = 250_000;
    pub const SIXTH_SECOND: useconds_t = 166_667;
    pub const EIGTHT_SECOND: useconds_t = 125_000;
    pub const TWELWTH_SECOND: useconds_t = 83_334;
    pub const SIXTEENTH_SECOND: useconds_t = 61_250;

    pub const IMMEDIATELY: useconds_t = 0;
}

const WIFI_SSID: &'static str = env!("NULED_WIFI_SSID");
const WIFI_PASSWORD: &'static str = env!("NULED_WIFI_PASSWORD");
const LED_COUNT: usize = 60;

#[no_mangle]
unsafe fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("NULED main() starting.");

    let peripherals = Peripherals::take().unwrap();
    let spi2 = peripherals.spi2.into_ref();

    let driver = SpiDriver::new_without_sclk(
        spi2,
        peripherals.pins.gpio8,
        Option::<AnyIOPin>::None,
        &spi::config::DriverConfig::new().dma(Dma::Auto(512)),
    ).unwrap();

    let spi_bus = SpiBusDriver::new(
        driver,
        &Config::new().baudrate(3_200.kHz().into()),
    ).unwrap();

    // LED writer
    let mut ws = Ws2812::new(spi_bus);

    log::info!("WS2812 driver started on SPI2 and GPIO8.");

    // WiFi starting
    led::blink(
        &mut ws,
        RGB8::new(0, 0, 255),
        RGB8::new(0, 0, 0),
        intervals::SIXTH_SECOND,
        intervals::SIXTH_SECOND,
        3,
    );

    //let lwip = netif::EspNetif::new(netif::NetifStack::Sta).unwrap();

    let event_loop = esp_idf_svc::eventloop::EspSystemEventLoop::take().unwrap();
    let non_volatile_storage = esp_idf_svc::nvs::EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = wifi::EspWifi::new(
        peripherals.modem,
        event_loop,
        Some(non_volatile_storage),
    ).unwrap();

    wifi_driver.set_configuration(&wifi::Configuration::Client(wifi::ClientConfiguration {
        ssid: WIFI_SSID.parse().unwrap(),
        password: WIFI_PASSWORD.parse().unwrap(),
        auth_method: wifi::AuthMethod::WPA2Personal,
        ..Default::default()
    })).unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();

    log::info!("Setup complete. Starting main program.");

    // Boot sequence finished
    led::fill(&mut ws, RGB8::new(0, 255, 0));
    usleep(intervals::SECOND);

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
    use crate::{intervals, LED_COUNT};
    use esp_idf_svc::hal::spi::{SpiBusDriver, SpiDriver};
    use esp_idf_svc::sys::{useconds_t, usleep};
    use smart_leds::hsv::{hsv2rgb, Hsv};
    use smart_leds::{SmartLedsWrite, RGB8};
    use ws2812_spi::Ws2812;

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