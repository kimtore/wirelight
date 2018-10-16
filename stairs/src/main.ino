// Configuration
#include "config.h"

// Use the FastLED library.
#define FASTLED_ALLOW_INTERRUPTS 0
#include <FastLED.h>

// Wifi and MQTT
#include <ESP8266WiFi.h>
#include <PubSubClient.h>

// Serial DEBUG mode on/off
#undef DEBUG_ANIMATION

#define LIGHT_ON "ON"
#define LIGHT_OFF "OFF"

// Digital IO pins.
#define PIN_LED 4

// Number of LEDs in the strip.
#define NUM_LEDS 60

#define ANIMATION_SPEED 50

// This variable holds the LED color state.
CRGB leds[NUM_LEDS];

// All the different modes available.
#define MAX_MODES 10
void modeBreathe();
void modeBreatheGradient();
void modeEase();
void modeGradient();
void modeJuggle();
void modeOff();
void modeRainbow();
void modeRainbowTrain();
void modeSolid();
void modeTemperature();

// Current state
struct {
    bool on;
    uint8_t brightness;
    uint8_t temperature;
    void (*effectFunc)();
    char* effect;
    uint8_t r;
    uint8_t g;
    uint8_t b;
} state;

// MQTT client handles
WiFiClient wifi_client;
PubSubClient mqtt_client(wifi_client);

// animate returns true if the animation should be stepped.
uint8_t animate(uint8_t speed) {
    static uint32_t lastClock = 0;
    uint32_t diff;
    uint32_t clock;
    uint32_t expectedDelay;

    clock = millis();
    diff = clock - lastClock;

    expectedDelay = map(speed, 1, 255, 250, 1);
    if (diff >= expectedDelay) {
        lastClock = clock;
        return 1;
    }

    return 0;
}

CRGB currentRGB() {
    return CRGB(state.r, state.g, state.b);
}

// animation animates from 0-255.
uint8_t animation(uint8_t speed) {
    static uint8_t an = 0;
    an += animate(speed);
    return an;
}

uint8_t ledAngle(uint8_t led) {
    return map(led, 0, NUM_LEDS-1, 0, 255);
}

// Switch off all LEDs.
void modeOff() {
    fill_solid(&leds[0], NUM_LEDS, CRGB::Black);
}

// Fill all LEDs with the same color. This mode does not have an additional parameter.
void modeSolid() {
    fill_solid(&leds[0], NUM_LEDS, currentRGB());
}

// Fill all LEDs with a solid color on the temperature scale.
void modeTemperature() {
    fill_solid(&leds[0], NUM_LEDS, HeatColor(state.temperature));
    FastLED.setBrightness(state.brightness);
}

// Animate the rainbow.
void modeRainbow() {
    uint8_t hue;
    for (uint8_t led = 0; led < NUM_LEDS; led++) {
        hue = ledAngle(led) + animation(ANIMATION_SPEED);
        leds[led] = CHSV(hue, 200, state.brightness);
    }
}

void mqtt_publish_state() {
    if (state.on) {
        mqtt_client.publish(MQTT_LIGHT_STATE_TOPIC, LIGHT_ON, true);
    } else {
        mqtt_client.publish(MQTT_LIGHT_STATE_TOPIC, LIGHT_OFF, true);
    }
}

void mqtt_handle_effect(const char *payload) {
    if (!strcmp(payload, "solid")) {
        state.effectFunc = modeSolid;
    } else if (!strcmp(payload, "temperature")) {
        state.effectFunc = modeTemperature;
    } else if (!strcmp(payload, "rainbow")) {
        state.effectFunc = modeRainbow;
    } else {
        return;
    }
    strcpy(state.effect, payload);
}

void mqtt_handle_command(const char *payload) {
    if (!strcmp(payload, LIGHT_ON)) {
        state.on = true;
        mqtt_handle_effect(state.effect);
    } else {
        state.on = false;
        state.effectFunc = modeOff;
    }
}

// function called when a MQTT message arrived
void mqtt_callback(char* p_topic, byte* p_payload, unsigned int p_length) {
    char msg[512];
    char payload[256];

    memcpy(payload, p_payload, 256);
    payload[p_length] = '\0';

    sprintf(msg, "Topic '%s' received payload: '%s'", p_topic, payload);
    Serial.println(msg);

    if (!strcmp(p_topic, MQTT_LIGHT_COMMAND_TOPIC)) {
        mqtt_handle_command(payload);
    }

    mqtt_publish_state();
}

void mqtt_connect() {
    Serial.println("INFO: Attempting MQTT connection...");
    if (mqtt_client.connect(MQTT_CLIENT_ID, MQTT_USER, MQTT_PASSWORD)) {
        Serial.println("INFO: connected");
        mqtt_publish_state();
        mqtt_client.subscribe(MQTT_LIGHT_COMMAND_TOPIC);
    } else {
        Serial.print("ERROR: failed, rc=");
        Serial.print(mqtt_client.state());
        Serial.println("DEBUG: try again in 5 seconds");
        delay(5000);
    }
}

// Initial setup, called once on boot.
void setup() {
    Serial.begin(115200);

    FastLED.addLeds<NEOPIXEL, PIN_LED>(leds, NUM_LEDS).setCorrection(TypicalSMD5050);

    memset(&state, sizeof state, 0);
    mqtt_handle_effect("solid");

    WiFi.begin(WIFI_SSID, WIFI_PASSWORD);

    mqtt_client.setServer(MQTT_SERVER_IP, MQTT_SERVER_PORT);
    mqtt_client.setCallback(mqtt_callback);
}

// Main loop. Make sure MQTT is connected, and render the LEDs.
void loop() {
    if (WiFi.status() == WL_CONNECTED) {
        if (!mqtt_client.connected()) {
            mqtt_connect();
            return;
        }
        mqtt_client.loop();
    }

    state.effectFunc();
    FastLED.show();
}
