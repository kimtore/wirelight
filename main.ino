/* Use the FastLED library instead of the Adafruit NeoPixel library. */
#include <FastLED.h>

/* This optional setting causes Encoder to use more optimized code,
 * It must be defined before Encoder.h is included. */
#define ENCODER_OPTIMIZE_INTERRUPTS
#include <Encoder.h>

/* Digital IO pins. */
#define PIN_ROTARY_ONE 3
#define PIN_ROTARY_TWO 2
#define PIN_BUTTON 4
#define PIN_LED 6

/* Number of LEDs in the strip. */
#define NUM_LEDS 60

/* Rotary encoder object */
Encoder encoder(PIN_ROTARY_ONE, PIN_ROTARY_TWO);

/* This variable holds the LED color state. */
CRGB leds[NUM_LEDS];

/* All the different modes that are available. */
#define MAX_MODES 2
void modeSolid();
void modeRainbow();
void (*modes[MAX_MODES])() = {
    &modeSolid,
    &modeRainbow
};

/* Define the different parameters that can be changed using the rotary wheel. */
#define PARAMETER_MODE 0
#define PARAMETER_HUE 1
#define PARAMETER_SATURATION 2
#define PARAMETER_VALUE 3
#define PARAMETER_SPEED 4
//#define PARAMETER_XXX 5
//#define PARAMETER_XXX 6
#define MAX_PARAMETERS 7

/* Holds the parameter that is currently changed using the rotary wheel. */
uint8_t activeParameter = PARAMETER_MODE;

/* Holds the values of the different parameters. */
uint8_t parameters[MAX_PARAMETERS] = { 0, 146, 255, 180, 128, 0, 0 };

/* Maximum values of the different parameters, plus one. Zero denotes that a
 * variable can stretch the full range of 0-255. */
uint8_t parameterMax[MAX_PARAMETERS] = { MAX_MODES, 0, 0, 0, 0, 0, 0 };

/* Rainbow colors, used to signify active parameter to the user. */
CRGB rainbowColors[MAX_PARAMETERS] = {
    CHSV(0,   255, 255),
    CHSV(21,  255, 255),
    CHSV(43,  255, 255),
    CHSV(85,  255, 255),
    CHSV(170, 255, 255),
    CHSV(198, 255, 255),
    CHSV(226, 255, 255)
};

/* Initial setup, called once on boot. */
void setup() {
    FastLED.addLeds<NEOPIXEL, PIN_LED>(leds, NUM_LEDS);
    pinMode(PIN_BUTTON, INPUT_PULLUP);
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

/* rotary_delta returns the number of full steps the rotary encoder has changed. */
int8_t rotary_delta() {
    int32_t e;
    int8_t rval = 0;

    e = encoder.read();

    rval = e / 4;
    e -= (rval * 4);

    encoder.write(e);

    return rval;
}

/* stepAnimation returns true if it is time for the next step in the animation,
 * according to the SPEED parameter. */
bool stepAnimation() {
    static uint32_t lastClock = 0;
    uint32_t diff;
    uint32_t clock;
    uint32_t expectedDelay;

    /* Animations are disabled when speed is zero */
    if (parameters[PARAMETER_SPEED] == 0) {
        return false;
    }

    clock = millis();
    diff = clock - lastClock;

    /* Otherwise, delay 1/SPEED milliseconds */
    lastClock = clock;
    expectedDelay = 1000 / parameters[PARAMETER_SPEED];

    return diff >= expectedDelay;
}

/* modeSolid draws a solid color across all LEDs, according to the parameters
 * HUE, SATURATION, VALUE. */
void modeSolid() {
    fill_solid(&leds[0], NUM_LEDS, CHSV(
        parameters[PARAMETER_HUE],
        parameters[PARAMETER_SATURATION],
        parameters[PARAMETER_VALUE]
    ));
}

/* modeRainbow draws an animated rainbow across all LEDs. */
void modeRainbow() {
    static uint8_t animation_hue = 0;
    uint8_t hue;
    uint8_t step;

    step = 255 / NUM_LEDS;

    for (uint8_t led = 0; led < NUM_LEDS; led++) {
        hue = animation_hue - parameters[PARAMETER_HUE] - (led * step);
        leds[led] = CHSV(hue, parameters[PARAMETER_SATURATION], parameters[PARAMETER_VALUE]);
    }

    if (!stepAnimation()) {
        return;
    }

    animation_hue--;
}

/* loop is called continuously. */
void loop() {

    /* Increase active parameter if button is pressed. */
    if (buttonChanged() && buttonPressed()) {
        activeParameter = (activeParameter + 1) % MAX_PARAMETERS;
        fill_solid(&leds[0], NUM_LEDS, rainbowColors[activeParameter]);
        FastLED.show();
    }

    /* Pause all activity while button is held down. */
    if (buttonPressed()) {
        return;
    }

    /* Increase or decrease parameter if rotary encoder moved. */
    parameters[activeParameter] += rotary_delta();
    parameters[activeParameter] %= parameterMax[activeParameter];

    /* Run the currently active mode. */
    modes[parameters[PARAMETER_MODE]]();
    FastLED.show();
}
