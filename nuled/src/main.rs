#![no_std]
#![no_main]

pub mod rust_mqtt;

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
use log::{debug, error, info, warn};
use smart_leds::hsv::Hsv;
use smart_leds::SmartLedsWrite;
use static_cell::StaticCell;
use ws2812_spi::prerendered::Ws2812;

const WIFI_SSID: &'static str = env!("NULED_WIFI_SSID");
const WIFI_PASSWORD: &'static str = env!("NULED_WIFI_PASSWORD");
const MQTT_SERVER: &'static str = env!("NULED_MQTT_SERVER");
const MQTT_PORT: u16 = 1883; //env!("NULED_MQTT_PORT");
const LED_COUNT: usize = 60;

static CLOCKS: StaticCell<Clocks> = StaticCell::new();
static NETWORK_STACK: StaticCell<embassy_net::Stack<esp_wifi::wifi::WifiDevice<'_, esp_wifi::wifi::WifiStaDevice>>> = StaticCell::new();
static NETWORK_STACK_MEMORY: StaticCell<embassy_net::StackResources<3>> = StaticCell::new();

const RX_BUFFER_SIZE: usize = 16384;
const TX_BUFFER_SIZE: usize = 16384;
static mut RX_BUFFER: [u8; RX_BUFFER_SIZE] = [0; RX_BUFFER_SIZE];
static mut TX_BUFFER: [u8; TX_BUFFER_SIZE] = [0; TX_BUFFER_SIZE];

#[main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger(log::LevelFilter::Debug);

    crate::info!("NULED booting.");

    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let clocks: &'static mut Clocks = CLOCKS.init(clocks);

    let embassy_timer = TimerGroup::new(peripherals.TIMG0, clocks);

    crate::info!("Initializing embassy...");
    esp_hal_embassy::init(clocks, embassy_timer.timer0);

    crate::debug!("Initializing WiFi configuration...");

    let wifi_timer = TimerGroup::new(peripherals.TIMG1, clocks);
    let wifi_init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Wifi,
        wifi_timer.timer0,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    ).unwrap();

    crate::debug!("Configuring WiFi for station mode.");

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

    spawner.must_spawn(ping_task());
    spawner.must_spawn(wifi_task(wifi_controller));
    spawner.must_spawn(net_task(network_stack));
    spawner.must_spawn(mqtt_task(network_stack));
    spawner.must_spawn(led_task(peripherals.SPI2, io.pins.gpio8, peripherals.DMA, clocks));

    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn mqtt_task(stack: &'static embassy_net::Stack<esp_wifi::wifi::WifiDevice<'static, esp_wifi::wifi::WifiStaDevice>>) {
    use embassy_net::tcp::TcpSocket;
    use embassy_time::Timer;
    use embassy_net::dns;

    // MQTT related imports
    use rust_mqtt::{
        client::{client::MqttClient, client_config::ClientConfig},
        utils::rng_generator::CountingRng,
    };

    loop {
        if !stack.is_link_up() {
            crate::warn!("Network is not up yet...");
            Timer::after_secs(1).await;
            continue;
        }

        let Some(config) = stack.config_v4() else {
            crate::warn!("Still waiting for IPv4 address...");
            Timer::after_secs(1).await;
            continue;
        };


        crate::info!("Acquired IPv4 address {:?}", config.address);

        let Ok(remote_ip) = stack.dns_query(MQTT_SERVER, dns::DnsQueryType::A).await.map(|x| x[0]) else {
            crate::warn!("DNS query failed for MQTT server, retrying in 30s...");
            Timer::after_secs(30).await;
            continue;
        };

        let mut sock = TcpSocket::new(
            stack,
            unsafe { &mut *core::ptr::addr_of_mut!(RX_BUFFER) },
            unsafe { &mut *core::ptr::addr_of_mut!(TX_BUFFER) },
        );

        if let Err(err) = sock.connect((remote_ip, MQTT_PORT)).await {
            crate::error!("Unable to connect to MQTT at {}:{}: {:?}", MQTT_SERVER, MQTT_PORT, err);
            continue;
        };

        const MQTT_BUFFER_SIZE: usize = 1024;

        let mut config = ClientConfig::new(
            rust_mqtt::client::client_config::MqttVersion::MQTTv5,
            CountingRng(20000),
        );
        config.add_max_subscribe_qos(rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1);
        config.add_client_id("clientId-8rhWgBODCl");
        config.max_packet_size = MQTT_BUFFER_SIZE as u32;
        let mut recv_buffer = [0; MQTT_BUFFER_SIZE];
        let mut write_buffer = [0; MQTT_BUFFER_SIZE];

        let mut client =
            MqttClient::<_, 5, _>::new(sock, &mut write_buffer, MQTT_BUFFER_SIZE, &mut recv_buffer, MQTT_BUFFER_SIZE, config);

        if let Err(err) = client.connect_to_broker().await {
            crate::error!("MQTT connection failed: {:?}", err);
            continue;
        }

        crate::info!("Connected to MQTT at {}:{}", MQTT_SERVER, MQTT_PORT);

        if let Err(err) = client.subscribe_to_topic("led/pallet/color/set").await {
            crate::error!("Unable to subscribe to {}: {:?}", "topic", err);
            continue;
        };

        loop {
            match client.receive_message().await {
                Ok((topic, data)) => {
                    crate::info!("MQTT receive on {}: {:?}", topic, data)
                }
                Err(err) => {
                    crate::error!("MQTT receive packet error: {:?}", err);
                    break;
                }
            }
        }
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

    crate::info!("WiFi task started.");

    loop {
        if let WifiState::StaConnected = get_wifi_state() {
            // already connected.
            // wait until we're no longer connected
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
            crate::info!("Starting WiFi controller...");
            wifi_controller.start().await.unwrap();
            crate::info!("WiFi started.");
        }

        crate::info!("WiFi connecting...");

        match wifi_controller.connect().await {
            Ok(_) => {
                crate::info!("WiFi connect success.");
            }
            Err(err) => {
                let msg = match err {
                    WifiError::NotInitialized => "not initialized",
                    WifiError::InternalError(_) => "internal error",
                    WifiError::Disconnected => "disconnected",
                    WifiError::UnknownWifiMode => "unknown wifi mode",
                };
                crate::error!("WiFi connect error: {}", msg);
                Timer::after(Duration::from_millis(5000)).await;
            }
        }
    }
}

#[embassy_executor::task]
async fn led_task(spi: SPI2, pin: GpioPin<8>, dma: esp_hal::peripherals::DMA, clocks: &'static Clocks<'static>) {
    crate::info!("LED task started.");
    crate::info!("Setting up DMA buffers.");

    let (
        tx_buffer,
        tx_descriptors,
        rx_buffer,
        rx_descriptors
    ) = esp_hal::dma_buffers!(512);

    let tx_dma = esp_hal::dma::DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();
    let rx_dma = esp_hal::dma::DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();

    crate::info!("Initializing SPI driver at 3.2MHz");

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

    crate::info!("Initializing SPI DMA bus...");

    let spi_driver = esp_hal::spi::master::SpiDmaBus::new(spi, tx_dma, rx_dma);

    let mut led_buffer = [0_u8; (LED_COUNT * 12) + 40];
    let mut ws = Ws2812::new(spi_driver, &mut led_buffer);

    crate::info!("WS2812 driver started on SPI2 and GPIO8.");

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
                //embassy_time::Timer::after_micros(1).await;
            }
            embassy_time::Timer::after_millis(50).await;
        }
    }
}

#[embassy_executor::task]
async fn ping_task() {
    use esp_wifi::current_millis;

    crate::info!("Ping task started.");

    let mut i = 0;
    let mut millis = current_millis();
    loop {
        i = i + 1;
        crate::info!("Ping {} +{}ms", i, current_millis()-millis);
        millis = current_millis();
        embassy_time::Timer::after_millis(500).await;
    }
}