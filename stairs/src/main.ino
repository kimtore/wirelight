// Configuration
#include "config.h"

// Use the FastLED library.
#define FASTLED_ALLOW_INTERRUPTS 0
#include <FastLED.h>

// Wifi and MQTT
#include <ESP8266WiFi.h>
#include <PubSubClient.h>

// Serial DEBUG mode on/off
#define DEBUG_SERIAL
#undef DEBUG_ANIMATION

#define LIGHT_ON "ON"
#define LIGHT_OFF "OFF"

// Digital IO pins.
#define PIN_LED 4

// Number of LEDs in the strip.
#define NUM_LEDS 60

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

// Set the order of modes in the Modeselektor.
void (*modes[MAX_MODES])() = {
    &modeOff,
    &modeTemperature,
    &modeSolid,
    &modeGradient,
    &modeEase,
    &modeRainbowTrain,
    &modeRainbow,
    &modeBreathe,
    &modeBreatheGradient,
    &modeJuggle
};

struct {
    bool on;
    uint8_t brightness;
    uint8_t temperature;
    char* effect;
    uint8_t r;
    uint8_t g;
    uint8_t b;
} state;

struct {
    uint8_t mode;
    uint8_t var;
    uint8_t hue;
    uint8_t sat;
    uint8_t val;
} settings;

// MQTT client handles
WiFiClient wifi_client;
PubSubClient mqtt_client(wifi_client);

// function called when a MQTT message arrived
void mqtt_callback(char* p_topic, byte* p_payload, unsigned int p_length) {
    char msg[512];
    char payload[256];

    memcpy(payload, p_payload, 256);
    payload[p_length] = '\0';

    sprintf(msg, "Topic '%s' received payload: '%s'", p_topic, payload);
    Serial.println(msg);

    if (!strcmp(payload, LIGHT_ON)) {
        state.on = true;
    } else {
        state.on = false;
    }
}


// Initial setup, called once on boot.
void setup() {
    Serial.begin(115200);

    FastLED.addLeds<NEOPIXEL, PIN_LED>(leds, NUM_LEDS).setCorrection(TypicalSMD5050);

    memset(&state, sizeof state, 0);

    fill_solid(&leds[0], NUM_LEDS, HeatColor(170));
    FastLED.setBrightness(100);
    FastLED.show();

    settings.mode = 3;
    settings.var  = 127;
    settings.hue  = 0;
    settings.sat  = 255;
    settings.val  = 127;

    WiFi.begin(WIFI_SSID, WIFI_PASSWORD);

    // init the MQTT connection
    mqtt_client.setServer(MQTT_SERVER_IP, MQTT_SERVER_PORT);
    mqtt_client.setCallback(mqtt_callback);
}

// animate returns true if the animation should be stepped.
uint8_t animate(uint8_t speed) {
    static uint32_t lastClock = 0;
    uint32_t diff;
    uint32_t clock;
    uint32_t expectedDelay;

    // Animations are disabled when speed is zero.
    if (settings.var == 0) {
        return 0;
    }

    clock = millis();
    diff = clock - lastClock;

    expectedDelay = map(speed, 1, 255, 250, 1);
    if (diff >= expectedDelay) {
        lastClock = clock;
        return 1;
    }

    return 0;
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
    fill_solid(&leds[0], NUM_LEDS, CHSV(settings.hue, settings.sat, settings.val));
}

// Fill all LEDs with a solid color on the temperature scale. The color
// breathes in and out to another temperature, defined by the extra parameter.
void modeTemperature() {
    uint8_t wave;
    uint8_t hue;

    wave = sin8(animation(230));
    hue = map8(wave, settings.hue, max(settings.var, settings.hue));

    fill_solid(&leds[0], NUM_LEDS, HeatColor(hue));
    FastLED.setBrightness(settings.val);
}

// Draw a linear gradient between two colors. The additional parameter defines
// the destination hue. The same saturation and value applies to both colors.
void modeGradient() {
    fill_gradient(
        &leds[0],
        0,          CHSV(settings.hue, settings.sat, settings.val),
        NUM_LEDS-1, CHSV(settings.var, settings.sat, settings.val)
    );
}

// Animate the rainbow. The hue parameter moves the rainbow back and forth,
// while the additional parameter regulates the animation speed.
void modeRainbow() {
    uint8_t hue;
    for (uint8_t led = 0; led < NUM_LEDS; led++) {
        hue = ledAngle(led) + animation(settings.var) + settings.hue;
        leds[led] = CHSV(hue, settings.sat, settings.val);
    }
}

// Fill all LEDs with a single color. Animate the intensity as a sine wave.
void modeBreathe() {
    uint8_t start;
    uint8_t sine;
    uint8_t value;

    sine = cubicwave8(animation(settings.var));
    start = settings.val - 100;
    if (start > settings.val) {
        start = 0;
    }
    value = map8(sine, start, settings.val);

#ifdef DEBUG_ANIMATION
    char buf[128];
    sprintf(buf, "sine:%4d  start:%4d  pot:%4d  value:%4d", sine, start, settings.val, value);
    Serial.println(buf);
    delay(20);
#endif

    fill_solid(&leds[0], NUM_LEDS, CHSV(settings.hue, settings.sat, value));
}

// Fill all LEDs with a single color, animating between two hues in a cubic
// ease in-out wave resembling a sine wave. The speed is calculated
// automatically based on distance on the color wheel.
void modeBreatheGradient() {
    uint8_t angle;
    uint8_t hue;
    uint8_t distance;
    uint8_t speed;

    distance = settings.var - settings.hue;
    speed = 255 - map8(distance, 15, 130);

    angle = cubicwave8(animation(speed));
    hue = map8(angle, settings.hue, settings.var);

    fill_solid(&leds[0], NUM_LEDS, CHSV(hue, settings.sat, settings.val));
}

// 1-N colored dots, weaving in and out of sync with each other. The variable
// adjusts the number of dots between 1 and half the LED strip.
//
// This function is adapted from https://github.com/atuline/FastLED-Demos.
void modeJuggle() {
    uint8_t led;
    uint8_t hue = settings.hue;
    uint8_t dots = map8(settings.var, 1, NUM_LEDS / 2);
    uint8_t step = 256 / dots;

    fadeToBlackBy(leds, NUM_LEDS, 20);

    for(uint8_t dot = 0; dot < dots; dot++) {
        led = beatsin16(dot+6, 0, NUM_LEDS-1);
        leds[led] |= CHSV(hue, settings.sat, settings.val);
        hue += step;
    }
}

// Animate each LED's intensity according to a cubic ease in-out function.
void modeEase() {
    uint8_t angle;
    uint8_t value;
    uint8_t an = animation(settings.var);
    for (uint8_t led = 0; led < NUM_LEDS; led++) {
        angle = map(led, 0, NUM_LEDS-1, 0, 255);
        value = ease8InOutCubic(sin8(angle + an));
        value = map(value, 0, 255, 0, settings.val);
        leds[led] = CHSV(settings.hue, settings.sat, value);
    }
}

// Animate a rainbow easing in and out. The hue parameter decides how far the
// rainbow will stretch on the color wheel, while the variable determines the speed.
void modeRainbowTrain() {
    uint8_t hue = 0;
    uint8_t angle;
    uint8_t value;
    uint8_t an = animation(settings.var);
    for (uint8_t led = 0; led < NUM_LEDS; led++) {
        angle = map(led, 0, NUM_LEDS-1, 0, 255);
        value = ease8InOutCubic(sin8(angle + an));
        hue = map8(value, 0, settings.hue);
        value = map8(value, 0, settings.val);
        leds[led] = CHSV(hue, settings.sat, value);
    }
}

void mqtt_publish_state() {
    if (state.on) {
        mqtt_client.publish(MQTT_LIGHT_STATE_TOPIC, LIGHT_ON, true);
    } else {
        mqtt_client.publish(MQTT_LIGHT_STATE_TOPIC, LIGHT_OFF, true);
    }
}

void mqtt_reconnect() {
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

// Read all potentiometers, and run one iteration of the active mode.
void loop() {
    if (WiFi.status() == WL_CONNECTED) {
        if (!mqtt_client.connected()) {
            mqtt_reconnect();
            return;
        }
        mqtt_client.loop();
    }

    modes[settings.mode]();
    if (state.on) {
        settings.val = 200;
    } else {
        settings.val = 0;
    }
    FastLED.show();
}
