#![no_std]
#![no_main]

pub mod rust_mqtt;
pub mod led;

use core::fmt::Write;
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
use esp_wifi::wifi::{WifiController};
use heapless::{spsc, String};
use smart_leds::SmartLedsWrite;
use static_cell::StaticCell;
use ws2812_spi::prerendered::Ws2812;

const WIFI_SSID: &'static str = env!("NULED_WIFI_SSID");
const WIFI_PASSWORD: &'static str = env!("NULED_WIFI_PASSWORD");
const MQTT_SERVER: &'static str = env!("NULED_MQTT_SERVER");
const MQTT_PORT: u16 = 1883; //env!("NULED_MQTT_PORT");
const MQTT_USERNAME: &'static str = env!("NULED_MQTT_USERNAME");
const MQTT_PASSWORD: &'static str = env!("NULED_MQTT_PASSWORD");

const LED_COUNT: usize = 60;

static CLOCKS: StaticCell<Clocks> = StaticCell::new();
static NETWORK_STACK: StaticCell<embassy_net::Stack<esp_wifi::wifi::WifiDevice<'_, esp_wifi::wifi::WifiStaDevice>>> = StaticCell::new();
static NETWORK_STACK_MEMORY: StaticCell<embassy_net::StackResources<3>> = StaticCell::new();
static COMMAND_QUEUE: StaticCell<spsc::Queue::<Command, 4>> = StaticCell::new();

const RX_BUFFER_SIZE: usize = 16384;
const TX_BUFFER_SIZE: usize = 16384;
static mut RX_BUFFER: [u8; RX_BUFFER_SIZE] = [0; RX_BUFFER_SIZE];
static mut TX_BUFFER: [u8; TX_BUFFER_SIZE] = [0; TX_BUFFER_SIZE];

enum Command {
    Fill(RGB),
}

#[main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger(log::LevelFilter::Info);

    info!("NULED booting.");

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
        spsc::Queue::<Command, 4>::new()
    );

    let (producer, consumer) = command_queue.split();

    spawner.must_spawn(wifi_task(wifi_controller));
    spawner.must_spawn(net_task(network_stack));
    spawner.must_spawn(mqtt_task(network_stack, producer));
    spawner.must_spawn(led_task(peripherals.SPI2, io.pins.gpio8, peripherals.DMA, clocks, consumer));

    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn mqtt_task(
    stack: &'static embassy_net::Stack<esp_wifi::wifi::WifiDevice<'static, esp_wifi::wifi::WifiStaDevice>>,
    mut queue: spsc::Producer<'static, Command, 4>,
) {
    use embassy_net::tcp::TcpSocket;
    use embassy_time::Timer;
    use embassy_net::dns;

    // MQTT related imports
    use rust_mqtt::{
        client::{client::MqttClient, client_config::ClientConfig},
        utils::rng_generator::CountingRng,
    };
    use rust_mqtt::packet::v5::publish_packet::QualityOfService;

    loop {
        if !stack.is_link_up() {
            warn!("Waiting for network...");
            Timer::after_secs(1).await;
            continue;
        }

        let Some(config) = stack.config_v4() else {
            warn!("Waiting for IPv4 address...");
            Timer::after_secs(1).await;
            continue;
        };


        info!("Acquired IPv4 address {}", config.address);

        let mqtt_server_ip = match stack.dns_query(MQTT_SERVER, dns::DnsQueryType::A).await {
            Err(err) => {
                warn!("DNS query failed for {}, retrying in 30s: {:?}", MQTT_SERVER, err);
                Timer::after_secs(30).await;
                continue;
            }
            Ok(ips) => ips[0],
        };

        let mut sock = TcpSocket::new(
            stack,
            unsafe { &mut *core::ptr::addr_of_mut!(RX_BUFFER) },
            unsafe { &mut *core::ptr::addr_of_mut!(TX_BUFFER) },
        );

        sock.set_keep_alive(Some(embassy_time::Duration::from_secs(15)));

        info!("Connecting to {} ({}) port {}...", MQTT_SERVER, mqtt_server_ip, MQTT_PORT);

        if let Err(err) = sock.connect((mqtt_server_ip, MQTT_PORT)).await {
            error!("Unable to connect to MQTT at {}:{}: {:?}", MQTT_SERVER, MQTT_PORT, err);
            Timer::after_secs(5).await;
            continue;
        };

        info!("Connected to MQTT, authenticating...");

        const MQTT_BUFFER_SIZE: usize = 1024;

        let mut config = ClientConfig::new(
            rust_mqtt::client::client_config::MqttVersion::MQTTv5,
            CountingRng(20000),
        );

        config.add_username(MQTT_USERNAME);
        config.add_password(MQTT_PASSWORD);
        config.add_max_subscribe_qos(QualityOfService::QoS1);
        config.add_client_id("ruled");
        config.max_packet_size = MQTT_BUFFER_SIZE as u32;

        let mut recv_buffer = [0; MQTT_BUFFER_SIZE];
        let mut write_buffer = [0; MQTT_BUFFER_SIZE];

        let mut client =
            MqttClient::<_, 5, _>::new(sock, &mut write_buffer, MQTT_BUFFER_SIZE, &mut recv_buffer, MQTT_BUFFER_SIZE, config);

        if let Err(err) = client.connect_to_broker().await {
            error!("MQTT authentication failed: {:?}", err);
            Timer::after_secs(5).await;
            continue;
        }

        info!("MQTT authenticated.");

        if let Err(err) = client.subscribe_to_topic("led/pallet/color/set").await {
            error!("Unable to subscribe to {}: {:?}", "topic", err);
            Timer::after_secs(5).await;
            continue;
        };

        info!("MQTT subscribed.");

        let mut rgb;

        loop {
            match client.receive_message().await {
                Ok((topic, data)) => {
                    debug!("MQTT receive on {}: {:?}", topic, data);

                    match RGB::parse(data) {
                        None => { continue; }
                        Some(value) => { rgb = value; }
                    };
                    info!("<-- R={}, G={}, B={}", rgb.r,rgb.g,rgb.b);
                    let _ = queue.enqueue(Command::Fill(rgb.clone()));
                }
                Err(err) => {
                    error!("MQTT receive packet error: {:?}", err);
                    break;
                }
            }

            let Some(rgb_string) = rgb.serialize() else {
                error!("Cannot serialize RGB string from {:?}", rgb);
                continue;
            };

            match client.send_message("led/pallet/color", rgb_string.as_bytes(), QualityOfService::QoS0, false).await {
                Ok(_) => info!("Published state"),
                Err(err) => {
                    error!("MQTT error publishing state: {:?}", err);
                    break;
                }
            };
        }
    }
}

#[derive(Debug, Clone)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl Default for RGB {
    fn default() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }
}

impl RGB {
    fn serialize(&self) -> Option<String<11>> {
        let mut s = String::new();
        write!(s, "{},{},{}", self.r, self.g, self.b).ok()?;
        Some(s)
    }

    fn parse_int_and_delimiter<'a>(mut iter: impl Iterator<Item=&'a u8>) -> Option<u8> {
        use core::str::FromStr;
        use heapless::String;
        let mut number_string = String::<3>::new();

        loop {
            let char = match iter.next() {
                None => None,
                Some(c) if *c as char == ',' => None,
                Some(c) => Some(*c as char),
            };

            match char {
                None => {
                    break;
                }
                Some(char) => {
                    if let Err(_) = number_string.push(char) {
                        return None;
                    };
                }
            }
        }

        u8::from_str(number_string.as_str()).ok()
    }

    fn parse(data: &[u8]) -> Option<Self> {
        let mut iter = data.iter();
        let r = Self::parse_int_and_delimiter(&mut iter)?;
        let g = Self::parse_int_and_delimiter(&mut iter)?;
        let b = Self::parse_int_and_delimiter(&mut iter)?;
        Some(Self { r, g, b })
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

#[embassy_executor::task]
async fn led_task(
    spi: SPI2,
    pin: GpioPin<8>,
    dma: esp_hal::peripherals::DMA,
    clocks: &'static Clocks<'static>,
    mut queue: spsc::Consumer<'static, Command, 4>,
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

    info!("Initializing SPI driver at 3.2MHz");

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

    info!("Initializing SPI DMA bus...");

    let spi_driver = esp_hal::spi::master::SpiDmaBus::new(spi, tx_dma, rx_dma);

    let mut led_buffer = [0_u8; (LED_COUNT * 12) + 40];
    let mut ws = Ws2812::new(spi_driver, &mut led_buffer);

    info!("WS2812 driver started on SPI2 and GPIO8.");


    loop {
        let Some(command) = queue.dequeue() else {
            embassy_time::Timer::after_millis(1).await;
            continue;
        };

        match command {
            Command::Fill(rgb) => {
                let color = smart_leds::RGB8::new(rgb.r, rgb.g, rgb.b);
                let data = [color; LED_COUNT];
                let data = smart_leds::brightness(
                    smart_leds::gamma(data.iter().cloned()),
                    127,
                );
                critical_section::with(|_| {
                    ws.write(data).unwrap();
                });
            }
        }
    }
}