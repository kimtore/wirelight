use crate::color::RGB;
use crate::rust_mqtt::{
    client::client_config::ClientConfig,
    utils::rng_generator::CountingRng,
};
use crate::rust_mqtt;
use crate::config::*;
use embassy_net::dns;
use embassy_net::tcp::TcpSocket;
use embassy_time::Duration;
use embassy_time::Timer;
use embedded_io_async::{Read, Write};
use heapless::{spsc, String};
use rand_core::RngCore;
use rust_mqtt::client::client::MqttClient;
use rust_mqtt::packet::v5::publish_packet::QualityOfService::*;
use crate::rust_mqtt::client::client_config::MqttVersion;
use core::fmt::Write as _;
use core::str::FromStr;
use crate::led::LedEffectParams;
use crate::mqtt::Error::MqttPublish;

const RX_BUFFER_SIZE: usize = 16384;
const TX_BUFFER_SIZE: usize = 16384;

static mut RX_BUFFER: [u8; RX_BUFFER_SIZE] = [0; RX_BUFFER_SIZE];
static mut TX_BUFFER: [u8; TX_BUFFER_SIZE] = [0; TX_BUFFER_SIZE];

struct MqttMessage<'a>(&'a [u8]);

impl MqttMessage<'_> {
    fn parse_rgb(&self) -> Option<RGB> {
        let s = core::str::from_utf8(self.0).ok()?;
        let mut parts= s.split(",");
        let r = parts.next()?;
        let g = parts.next()?;
        let b = parts.next()?;
        Some(RGB {
            r: f32::from_str(r).ok()?,
            g: f32::from_str(g).ok()?,
            b: f32::from_str(b).ok()?,
        })
    }

    fn parse_effect(&self) -> Option<Effect> {
        let effect_str = core::str::from_utf8(self.0).ok()?;
        match effect_str {
            "rainbow" => Some(Effect::Rainbow),
            "solid" => Some(Effect::Solid),
            "polyrhythm" => Some(Effect::Polyrhythm),
            "gradient" => Some(Effect::Gradient),
            _ => None,
        }
    }

    fn parse_float(&self) -> Option<f32> {
        let s = core::str::from_utf8(self.0).ok()?;
        f32::from_str(s).ok()
    }
}

enum MqttResponse {
    RGB(RGB),
    Effect(Effect),
    Number(f32),
}

impl MqttResponse {
    fn serialize<'a>(self) -> Option<String<32>> {
        let mut s = String::new();
        match self {
            MqttResponse::RGB(rgb) => {
                write!(s, "{},{},{}", rgb.r, rgb.g, rgb.b).ok()?;
            }
            MqttResponse::Effect(effect) => {
                s.write_str(match effect {
                    Effect::Solid => "solid",
                    Effect::Rainbow => "rainbow",
                    Effect::Polyrhythm => "polyrhythm",
                    Effect::Gradient => "gradient",
                }).ok()?;
            }
            MqttResponse::Number(num) => {
                let mut buf = ryu::Buffer::new();
                s.write_str(buf.format(num)).ok()?;
            }
        }
        Some(s)
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct ServerState {
    effect: Effect,
    led_effect_params: LedEffectParams,
}

#[derive(Debug, Default, Copy, Clone)]
pub enum Effect {
    Solid,
    #[default]
    Rainbow,
    Gradient,
    Polyrhythm,
}

#[derive(Debug)]
pub enum LedEffectCommand {
    ChangeEffect(Effect),
    ConfigureParams(LedEffectParams),
}

enum Error {
    MqttReceive(rust_mqtt::packet::v5::reason_codes::ReasonCode),
    MqttPublish(rust_mqtt::packet::v5::reason_codes::ReasonCode),
    InvalidTopic,
    ParseParameter,
    Serialize,
}

#[embassy_executor::task]
pub async fn mqtt_task(
    stack: &'static embassy_net::Stack<esp_wifi::wifi::WifiDevice<'static, esp_wifi::wifi::WifiStaDevice>>,
    mut queue: spsc::Producer<'static, LedEffectCommand, 16>,
) {
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

        sock.set_timeout(Some(Duration::from_secs(30)));
        sock.set_keep_alive(Some(Duration::from_secs(15)));

        info!("Connecting to {} ({}) port {}...", MQTT_SERVER, mqtt_server_ip, MQTT_PORT);

        if let Err(err) = sock.connect((mqtt_server_ip, MQTT_PORT)).await {
            error!("Unable to connect to MQTT at {}:{}: {:?}", MQTT_SERVER, MQTT_PORT, err);
            Timer::after_secs(5).await;
            continue;
        };

        info!("Connected to MQTT, authenticating...");

        const MQTT_BUFFER_SIZE: usize = 1024;

        let mut config = ClientConfig::new(
            MqttVersion::MQTTv5,
            CountingRng(20000),
        );

        config.add_username(MQTT_USERNAME);
        config.add_password(MQTT_PASSWORD);
        config.add_max_subscribe_qos(QoS1);
        config.add_client_id("ruled");
        config.max_packet_size = MQTT_BUFFER_SIZE as u32;
        config.keep_alive = 3600;

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

        if let Err(err) = client.subscribe_to_topic("led/pallet/+/set").await {
            error!("Unable to subscribe to {}: {:?}", "topic", err);
            Timer::after_secs(5).await;
            continue;
        };

        info!("MQTT subscribed.");

        let mut state = ServerState::default();

        loop {
            let Err(err) = mqtt_process_message(&mut client, &mut state, &mut queue).await else {
                continue;
            };

            match err {
                Error::MqttReceive(err) => {
                    error!("MQTT receive packet error: {:?}", err);
                    break;
                }
                Error::MqttPublish(err) => {
                    error!("MQTT publish packet error: {:?}", err);
                    break;
                }
                Error::InvalidTopic => {
                    debug!("MQTT received data on unrecognized topic");
                }
                Error::ParseParameter => {
                    error!("Unable to parse color or effect parameter from MQTT");
                }
                Error::Serialize => {
                    error!("RGB serialization failed");
                }
            }
        }
    }
}

/// Receive a valid message over any of the configured MQTT topics, configure LEDs based on that,
/// and report back the current state.
async fn mqtt_process_message<'a, T, const MAX_PROPERTIES: usize, R>(
    client: &mut MqttClient<'a, T, MAX_PROPERTIES, R>,
    state: &mut ServerState,
    queue: &mut spsc::Producer<'static, LedEffectCommand, 16>,
) -> Result<(), Error>
where
    T: Read + Write,
    R: RngCore,
{
    use Error::*;

    let (topic, data) = client.receive_message().await.map_err(MqttReceive)?;

    debug!("MQTT receive on {}: {:?}", topic, data);

    let message = MqttMessage(data);

    match topic {
        "led/pallet/color1/set" => {
            state.led_effect_params.color1 = message.parse_rgb().ok_or(ParseParameter)?;
        }
        "led/pallet/color2/set" => {
            state.led_effect_params.color2 = message.parse_rgb().ok_or(ParseParameter)?;
        }
        "led/pallet/chroma/set" => {
            state.led_effect_params.chroma = message.parse_float().ok_or(ParseParameter)?;
        }
        "led/pallet/luminance/set" => {
            state.led_effect_params.luminance = message.parse_float().ok_or(ParseParameter)?;
        }
        "led/pallet/speed/set" => {
            state.led_effect_params.speed = message.parse_float().ok_or(ParseParameter)?;
        }
        "led/pallet/size/set" => {
            state.led_effect_params.size = message.parse_float().ok_or(ParseParameter)?;
        }
        "led/pallet/effect/set" => {
            state.effect = message.parse_effect().ok_or(ParseParameter)?;
            let _ = queue.enqueue(LedEffectCommand::ChangeEffect(state.effect));
        }
        _ => return Err(InvalidTopic)
    };

    let _ = queue.enqueue(LedEffectCommand::ConfigureParams(state.led_effect_params));
    info!("Update: {:?}", state);

    mqtt_publish_state(client, "led/pallet/color1", MqttResponse::RGB(state.led_effect_params.color1)).await?;
    mqtt_publish_state(client, "led/pallet/color2", MqttResponse::RGB(state.led_effect_params.color2)).await?;
    mqtt_publish_state(client, "led/pallet/effect", MqttResponse::Effect(state.effect)).await?;
    mqtt_publish_state(client, "led/pallet/chroma", MqttResponse::Number(state.led_effect_params.chroma)).await?;
    mqtt_publish_state(client, "led/pallet/luminance", MqttResponse::Number(state.led_effect_params.luminance)).await?;
    mqtt_publish_state(client, "led/pallet/size", MqttResponse::Number(state.led_effect_params.size)).await?;
    mqtt_publish_state(client, "led/pallet/speed", MqttResponse::Number(state.led_effect_params.speed)).await?;

    Ok(())
}

async fn mqtt_publish_state<'a, T, const MAX_PROPERTIES: usize, R>(
    client: &mut MqttClient<'a, T, MAX_PROPERTIES, R>,
    topic: &'static str,
    payload: MqttResponse,
) -> Result<(), Error>
where
    T: Read + Write,
    R: RngCore,
{
    let payload = payload.serialize().ok_or(Error::Serialize)?;
    client.send_message(topic, payload.as_bytes(), QoS0, false).await.map_err(MqttPublish)
}