// Configuration
#include "config.h"

// see https://github.com/FastLED/FastLED/issues/306
#define FASTLED_INTERRUPT_RETRY_COUNT 0
//#define FASTLED_ALLOW_INTERRUPTS 0

// Use the FastLED library.
#include <FastLED.h>

// Wifi library
#include <ESP8266WiFi.h>

#define LIGHT_ON "ON"
#define LIGHT_OFF "OFF"

#define SERIAL_SPEED 115200

// Digital IO pin where the LED strip is connected.
// Pin 4 is also known as D4 or GPIO2.
#define PIN_LED 4

// Number of LEDs in the strip.
#define NUM_LEDS 59

// Max power usage for the entire strip, in milliwatts
#define MAX_POWER 1000

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
    uint16_t mired;
    char effect[64];
    CRGB rgb;
} state;

// WIFI
WiFiClient wifi_client;

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
    fill_solid(&leds[0], NUM_LEDS, state.rgb);
}

// Fill all LEDs with a solid color on the temperature scale.
void modeTemperature() {
    fill_solid(&leds[0], NUM_LEDS, HeatColor(state.temperature));
}

// Animate the rainbow.
void modeRainbow() {
    uint8_t hue;
    for (uint8_t led = 0; led < NUM_LEDS; led++) {
        hue = ledAngle(led) + animation(ANIMATION_SPEED);
        leds[led] = CHSV(hue, 200, state.brightness);
    }
}

void mqtt_handle_rgb(const char *payload) {
    char rgbStr[16];
    char *ptr;
    CRGB color;

    strncpy(rgbStr, payload, 15);

    ptr = strtok(rgbStr, ",");
    if (ptr == NULL) {
        return;
    }
    color.r = atoi(ptr);

    ptr = strtok(NULL, ",");
    if (ptr == NULL) {
        return;
    }
    color.g = atoi(ptr);

    ptr = strtok(NULL, ",");
    if (ptr == NULL) {
        return;
    }
    color.b = atoi(ptr);

    state.rgb = color;
}

void set_brightness(uint8_t brightness) {
    state.brightness = brightness;
    Serial.printf("Brightness changed to %d\n", state.brightness);
}

void set_temperature(uint16_t mired) {
    state.mired = mired;
    state.temperature = map(state.mired, 500, 153, 125, 255);
    state.rgb = HeatColor(state.temperature);
    Serial.printf("Temperature changed to mired/%d fastled/%d\n", state.mired, state.temperature);
}

// Initial setup, called once on boot.
void setup() {
    Serial.begin(SERIAL_SPEED);

    FastLED.addLeds<NEOPIXEL, PIN_LED>(leds, NUM_LEDS).setCorrection(TypicalSMD5050);
    set_max_power_in_milliwatts(MAX_POWER);

    memset(&state, sizeof state, 0);

    WiFi.begin(WIFI_SSID, WIFI_PASSWORD);
    WiFi.setSleepMode(WIFI_NONE_SLEEP);

    set_brightness(255);
    set_temperature(2000);
}

// Main loop. Make sure MQTT is connected, and render the LEDs.
void loop() {
    if (WiFi.status() == WL_CONNECTED) {
    }

    set_temperature(state.mired + 1);
    if (state.mired > 8000) {
        state.mired = 2000;
    }
    fill_solid(&leds[0], NUM_LEDS, state.rgb);
    FastLED.setBrightness(state.brightness);
    FastLED.show();
}
