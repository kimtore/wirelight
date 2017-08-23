/* Use the FastLED library instead of the Adafruit NeoPixel library. */
#include <FastLED.h>

/* This optional setting causes Encoder to use more optimized code,
 * It must be defined before Encoder.h is included. */
#define ENCODER_OPTIMIZE_INTERRUPTS
#include <Encoder.h>

/* Digital output pins. */
#define PIN_ROTARY_LEFT 2
#define PIN_ROTARY_RIGHT 3
#define PIN_BUTTON 4
#define PIN_LED 6

/* Number of LEDs in the strip. */
#define NUM_LEDS 60

/* Rotary encoder object */
Encoder encoder(PIN_ROTARY_LEFT, PIN_ROTARY_RIGHT);

/* This variable holds the LED color state. */
CRGB leds[NUM_LEDS];

/* Define the different variables that can be changed using the rotary wheel. */
#define VARIABLE_MODE 0
#define VARIABLE_HUE 1
#define VARIABLE_SATURATION 2
#define VARIABLE_VALUE 3
#define VARIABLE_SPEED 4
//#define VARIABLE_XXX 5
//#define VARIABLE_XXX 6
#define MAX_VARIABLES 7

/* Holds the variable that is currently changed using the rotary wheel. */
uint8_t activeVariable = 0;

/* Rainbow colors, used to signify active variable to the user. */
CRGB rainbowColors[] = {
    CHSV(0,   255, 255),
    CHSV(36,  255, 255),
    CHSV(73,  255, 255),
    CHSV(109, 255, 255),
    CHSV(146, 255, 255),
    CHSV(182, 255, 255),
    CHSV(219, 255, 255)
};

void setup() {
    FastLED.addLeds<NEOPIXEL, PIN_LED>(leds, NUM_LEDS);

    pinMode(PIN_BUTTON,       INPUT_PULLUP);
    pinMode(PIN_ROTARY_LEFT,  INPUT_PULLUP);
    pinMode(PIN_ROTARY_RIGHT, INPUT_PULLUP);
}

/* buttonPressed returns true if the button is pressed, false otherwise. */
uint8_t buttonPressed() {
    return digitalRead(PIN_BUTTON) == LOW;
}

/* buttonChanged returns true if the button was pressed or depressed since last
 * time the function was called. */
bool buttonChanged() {
    static bool lastState = false;
    bool state;
    bool rval;

    state = buttonPressed();
    rval = (state != lastState);
    lastState = state;

    return rval;
}

void loop() {
    int32_t e;

    if (buttonChanged()) {
        if (buttonPressed()) {
            activeVariable = (activeVariable + 1) % MAX_VARIABLES;
            fill_solid(&leds[0], NUM_LEDS, rainbowColors[activeVariable]);
        } else {
            fill_solid(&leds[0], NUM_LEDS, CRGB::Black);
        }
    }

    e = encoder.read();
    if (e <= -4) {
        leds[10] = CHSV(0, 255, 255);
        e += 4;
    } else if (e >= 4) {
        leds[10] = CHSV(120, 255, 255);
        e -= 4;
    } else {
        leds[10] = CRGB::Black;
    }
    encoder.write(e);

    FastLED.show();
}
