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
#define MAX_MODES 4
void modeTemperature();
void modeSolid();
void modeSolidRainbow();
void modeRainbow();
void (*modes[MAX_MODES])() = {
    &modeTemperature,
    &modeSolid,
    &modeSolidRainbow,
    &modeRainbow
};

/* Define the different parameters that can be changed using the rotary wheel. */
#define PARAMETER_MODE 0
#define PARAMETER_HUE 1
#define PARAMETER_SATURATION 2
#define PARAMETER_VALUE 3
#define PARAMETER_SPEED 4
#define PARAMETER_POSITION 5
#define PARAMETER_SIZE 6
#define MAX_PARAMETERS 7

/* Holds the parameter that is currently changed using the rotary wheel. */
uint8_t activeParameter = PARAMETER_MODE;

/* Holds the values of the different parameters. */
uint8_t parameters[MAX_PARAMETERS] = { 0, 190, 255, 128, 10, 0, NUM_LEDS-1 };

/* Maximum values of the different parameters, plus one. Zero denotes that a
 * variable can stretch the full range of 0-255. */
uint8_t parameterMax[MAX_PARAMETERS] = { MAX_MODES, 0, 0, 0, 0, NUM_LEDS, NUM_LEDS };

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
    FastLED.addLeds<NEOPIXEL, PIN_LED>(leds, NUM_LEDS).setCorrection(TypicalSMD5050);
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
    bool rval;

    /* Animations are disabled when speed is zero */
    if (parameters[PARAMETER_SPEED] == 0) {
        return false;
    }

    clock = millis();
    diff = clock - lastClock;

    /* Otherwise, delay 1000/SPEED milliseconds. */
    expectedDelay = 1000 / parameters[PARAMETER_SPEED];
    rval = (diff >= expectedDelay);

    if (rval) {
        lastClock = clock;
    }

    return rval;
}

/* ledRange returns a array of bools indicating which LEDs should be drawn,
 * according to the start and end parameters. This function differs from the
 * FastLED library function in the sense that end may be smaller than start. */
bool *ledRange(uint8_t start, uint8_t end) {
    static bool lit[NUM_LEDS];

    memset(&lit, 0, sizeof lit);

    start %= NUM_LEDS;
    end %= NUM_LEDS;

    if (end >= start) {
        /* Contiguous block */

        for (uint8_t led = start; led <= end; led++) {
            lit[led] = true;
        }
    } else {
        /* Split into two parts. Draw start first, then end. */

        for (uint8_t led = 0; led <= end; led++) {
            lit[led] = true;
        }

        for (uint8_t led = start; led < NUM_LEDS; led++) {
            lit[led] = true;
        }
    }

    return lit;
}

/* Fill LEDs with a solid color according to the specified range. All other
 * LEDs are blacked out. */
void fillSolid(bool *range, CRGB color) {
    fill_solid(&leds[0], NUM_LEDS, CRGB::Black);

    for (uint8_t led = 0; led < NUM_LEDS; led++) {
        if (range[led]) {
            leds[led] = color;
        }
    }
}

/* animatedLEDs returns a boolean array of LEDs that should be lit up according
* to POSITION and SIZE. Additionally, the movement animation is attempted. If an
* animation occurred, the animated boolean is set to true. Otherwise, it is set
* to false. If the animated parameter is NULL, it is ignored. */
bool *animatedLEDs(bool *animated = NULL) {
    static uint8_t animation_start = 0;
    uint8_t start;
    bool *range;
    bool anim;

    start = (animation_start + parameters[PARAMETER_POSITION]) % NUM_LEDS;
    range = ledRange(start, start + parameters[PARAMETER_SIZE]);
    anim = stepAnimation();

    if (anim) {
        animation_start++;
        animation_start %= NUM_LEDS;
    }

    if (animated) {
        *animated = anim;
    }

    return range;
}

/* modeTemperature draws a solid color across all LEDs, according to color temperature. */
void modeTemperature() {
    bool *range;

    range = animatedLEDs();
    fillSolid(range, HeatColor(parameters[PARAMETER_HUE]));

    FastLED.setBrightness(parameters[PARAMETER_VALUE]);
}

/* modeSolid draws a solid color across all LEDs, according to the parameters
 * HUE, SATURATION, VALUE. */
void modeSolid() {
    bool *range;

    range = animatedLEDs();
    fillSolid(range, CHSV(
        parameters[PARAMETER_HUE],
        parameters[PARAMETER_SATURATION],
        parameters[PARAMETER_VALUE]
    ));
}

/* modeSolidRainbow draws a solid color across all LEDs, animating it according
 * to the color wheel. */
void modeSolidRainbow() {
    static uint8_t hue = 0;
    bool animated;
    bool *range;

    range = animatedLEDs(&animated);

    fillSolid(range, CHSV(
        hue + parameters[PARAMETER_HUE],
        parameters[PARAMETER_SATURATION],
        parameters[PARAMETER_VALUE]
    ));

    if (animated) {
        hue++;
    }
}

/* modeRainbow draws an animated rainbow across all LEDs. */
void modeRainbow() {
    static uint8_t animation_hue = 0;
    uint8_t hue;
    uint8_t step;
    uint8_t steps = 0;
    bool animated;
    bool *range;

    range = animatedLEDs(&animated);

    /* Increment the color wheel so that it matches the number of visible LEDs */
    step = 255 / parameters[PARAMETER_SIZE];

    /* Black out all LEDs */
    fill_solid(&leds[0], NUM_LEDS, CRGB::Black);

    for (uint8_t led = 0; led < NUM_LEDS; led++) {
        if (!range[led]) {
            continue;
        }
        hue = animation_hue + parameters[PARAMETER_HUE] + (steps++ * step);
        leds[led] = CHSV(hue, parameters[PARAMETER_SATURATION], parameters[PARAMETER_VALUE]);
    }

    if (animated) {
        animation_hue++;
    }
}

/* Check if rotary wheel changed, and adjust active parameter */
void adjustParameter() {
    uint8_t changed;
    int8_t delta;
    uint8_t *mx = &parameterMax[activeParameter];

    delta = rotary_delta();
    changed = parameters[activeParameter] + delta;

    /* Wrap around */
    if (*mx > 0 && changed >= *mx) {
        if (delta < 1) {
            changed = *mx - 1;
        } else {
            changed = 0;
        }
    }

    parameters[activeParameter] = changed;
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

    /* Adjust active parameter, if neccessary */
    adjustParameter();

    /* Increase or decrease parameter if rotary encoder moved. */
    /* Run the currently active mode. */
    modes[parameters[PARAMETER_MODE]]();
    FastLED.show();
}
