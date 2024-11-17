pub const WIFI_SSID: &'static str = env!("NULED_WIFI_SSID");
pub const WIFI_PASSWORD: &'static str = env!("NULED_WIFI_PASSWORD");
pub const MQTT_SERVER: &'static str = env!("NULED_MQTT_SERVER");
pub const MQTT_PORT: u16 = must_parse_u16(env!("NULED_MQTT_PORT"));
pub const MQTT_USERNAME: &'static str = env!("NULED_MQTT_USERNAME");
pub const MQTT_PASSWORD: &'static str = env!("NULED_MQTT_PASSWORD");
pub const LED_COUNT: usize = must_parse_led_count(env!("NULED_LED_COUNT")) as usize;

const fn must_parse_u16(s: &str) -> u16 {
    match u16::from_str_radix(s, 10) {
        Ok(val) => val,
        Err(_) => panic!("value is not a number"),
    }
}

const fn must_parse_led_count(s: &str) -> usize {
    match must_parse_u16(s) {
        0 => panic!("LED count must be greater than zero"),
        x => x as usize,
    }
}