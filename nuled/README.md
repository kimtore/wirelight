# NULED
NULED is a IoT WS2812 LED controller for the ESP32C3 chip, written in Rust.

NULED connects to WiFi and is configured via MQTT. It supports numerous effects,
and does its best to perform accurate color gradients.

The WS2812 LEDs are driven using the SPI2 controller using DMA, and the signal
output is on the GPIO8 port.

Notable crate dependencies: `esp-hal` family for hardware interfaces, `embassy` for async and network support,
`smoltcp` as the network stack, and `ws2812-spi` and `smart-leds` as the WS2812 driver.

## Constraints
* WiFi: only WPA2 and WPA3 are supported.
* MQTT: traffic is not encrypted, and server must support MQTTv5.

## Configuration

Copy the `config.example` and edit the values to fit your environment.

## Developing

To compile this project, apply the required configuration as noted above, and run:

```shell
rustup target add riscv32imc-unknown-none-elf
cargo build --release
espflash flash --monitor target/riscv32imc-unknown-none-elf/release/nuled
```

To flash your ESP32 chip:

```shell
espflash flash --monitor target/riscv32imc-unknown-none-elf/release/nuled
```