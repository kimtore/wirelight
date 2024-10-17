#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_net::IpListenEndpoint;
use embassy_net::udp::{PacketMetadata};
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
use log::{error, info, warn};
use smart_leds::hsv::Hsv;
use smart_leds::SmartLedsWrite;
use static_cell::StaticCell;
use ws2812_spi::prerendered::Ws2812;

const WIFI_SSID: &'static str = env!("NULED_WIFI_SSID");
const WIFI_PASSWORD: &'static str = env!("NULED_WIFI_PASSWORD");
const LED_COUNT: usize = 60;

static CLOCKS: StaticCell<Clocks> = StaticCell::new();
static NETWORK_STACK: StaticCell<embassy_net::Stack<esp_wifi::wifi::WifiDevice<'_, esp_wifi::wifi::WifiStaDevice>>> = StaticCell::new();
static NETWORK_STACK_MEMORY: StaticCell<embassy_net::StackResources<3>> = StaticCell::new();

const RX_BUFFER_SIZE: usize = 16384;
const TX_BUFFER_SIZE: usize = 16384;
static mut RX_METADATA_BUFFER: [PacketMetadata; 32] = [PacketMetadata::EMPTY; 32];
static mut TX_METADATA_BUFFER: [PacketMetadata; 32] = [PacketMetadata::EMPTY; 32];
static mut RX_BUFFER: [u8; RX_BUFFER_SIZE] = [0; RX_BUFFER_SIZE];
static mut TX_BUFFER: [u8; TX_BUFFER_SIZE] = [0; TX_BUFFER_SIZE];

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

    log::debug!("Initializing WiFi configuration...");

    let wifi_timer = TimerGroup::new(peripherals.TIMG1, clocks);
    let wifi_init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Wifi,
        wifi_timer.timer0,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    ).unwrap();

    log::debug!("Configuring WiFi for station mode.");

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
    spawner.must_spawn(udp_receive_task(network_stack));
    spawner.must_spawn(led_task(peripherals.SPI2, io.pins.gpio8, peripherals.DMA, clocks));

    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn udp_receive_task(stack: &'static embassy_net::Stack<esp_wifi::wifi::WifiDevice<'static, esp_wifi::wifi::WifiStaDevice>>) {
    use embassy_net::udp::UdpSocket;
    use embassy_time::Timer;

    loop {
        if !stack.is_link_up() {
            warn!("Network is not up yet...");
            Timer::after_secs(1).await;
            continue;
        }

        let Some(config) = stack.config_v4() else {
            warn!("Still waiting for IPv4 address...");
            Timer::after_secs(1).await;
            continue;
        };


        info!("Acquired IPv4 address {:?}", config.address);

        let mut sock = UdpSocket::new(
            stack,
            unsafe { &mut *core::ptr::addr_of_mut!(RX_METADATA_BUFFER) },
            unsafe { &mut *core::ptr::addr_of_mut!(RX_BUFFER) },
            unsafe { &mut *core::ptr::addr_of_mut!(TX_METADATA_BUFFER) },
            unsafe { &mut *core::ptr::addr_of_mut!(TX_BUFFER) },
        );

        if let Err(err) = sock.bind(IpListenEndpoint::from(56700)) {
            error!("bind to port 56700: {:?}", err);
            continue;
        }

        const RECV_BUF_SIZE: usize = 1024;
        let mut data_receive_buffer: [u8; RECV_BUF_SIZE] = [0; RECV_BUF_SIZE];

        loop {
            match sock.recv_from(&mut data_receive_buffer).await {
                Ok((length, endpoint)) => {
                    info!("Received packet from {:?} of size {}", endpoint, length);
                    let data = &data_receive_buffer[0..length];
                    handle_udp_packet(data);
                    sock.send_to(data, endpoint).await.unwrap();
                }
                Err(err) => {
                    error!("error receiving UDP packet: {:?}", err);
                    break;
                }
            }
        }
    }
}

fn handle_udp_packet(data: &[u8]) {
    info!("--> {:?}", data);
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

    log::info!("WiFi task started.");

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
            log::info!("Starting WiFi controller...");
            wifi_controller.start().await.unwrap();
            log::info!("WiFi started.");
        }

        log::info!("WiFi connecting...");

        match wifi_controller.connect().await {
            Ok(_) => {
                log::info!("WiFi connect success.");
            }
            Err(err) => {
                log::error!("WiFi connect error: {:?}", err);
                Timer::after(Duration::from_millis(5000)).await;
            }
        }
    }
}

#[embassy_executor::task]
async fn led_task(spi: SPI2, pin: GpioPin<8>, dma: esp_hal::peripherals::DMA, clocks: &'static Clocks<'static>) {
    log::info!("LED task started.");
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
                //embassy_time::Timer::after_micros(1).await;
            }
            embassy_time::Timer::after_millis(50).await;
        }
    }
}

#[embassy_executor::task]
async fn ping_task() {
    use esp_wifi::current_millis;

    log::info!("Ping task started.");

    let mut i = 0;
    let mut millis = current_millis();
    loop {
        i = i + 1;
        log::info!("Ping {i} +{}ms", current_millis()-millis);
        millis = current_millis();
        embassy_time::Timer::after_millis(500).await;
    }
}