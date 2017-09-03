// Use the FastLED library.
#include <FastLED.h>

// Serial DEBUG mode on/off
#undef DEBUG_SERIAL
#undef DEBUG_ANIMATION
#undef DEBUG_POTS

// Analog input pins.
//
// .-------.
// |  O O  |
// | O O O |
// '-------'
//
// The device has five potentiometer inputs. The top left is a switch with 10
// different values, while the other four have a variable input between 0-1023.
//
// The switch defines how the LEDs behave. The top button adjusts the parameter
// in the current mode. The bottom three potentiometers adjusts hue, saturation,
// and value for all the LEDs.
#define PIN_POT_TOP 1
#define PIN_POT_LEFT 2
#define PIN_POT_CENTER 3
#define PIN_POT_RIGHT 4
#define PIN_POT_SWITCH 5

// Digital IO pins.
#define PIN_LED 5

// Number of LEDs in the strip.
#define NUM_LEDS 30

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
    uint8_t mode;
    uint8_t var;
    uint8_t hue;
    uint8_t sat;
    uint8_t val;
} pots;

// Initial setup, called once on boot.
void setup() {
    FastLED.addLeds<NEOPIXEL, PIN_LED>(leds, NUM_LEDS).setCorrection(TypicalSMD5050);
#ifdef DEBUG_SERIAL
    Serial.begin(115200);
#endif
}

#ifdef DEBUG_POTS
void debugPots() {
    uint8_t i;
    uint16_t analog;
    char buf[128];
    char *ptr = &buf[0];

    ptr += sprintf(ptr, "adc(");
    for (i=1; i<6; i++) {
        analog = analogRead(i);
        ptr += sprintf(ptr, "%d: %4d, ", i, analog);
    }
    ptr += sprintf(ptr, ") ");
    ptr += sprintf(ptr, "pots(mode:%d, var:%3d, hue:%3d, sat:%3d, val:%3d)", pots.mode, pots.var, pots.hue, pots.sat, pots.val);

    *ptr = '\0';
    Serial.println(buf);
}
#endif

// animate returns true if the animation should be stepped.
uint8_t animate(uint8_t speed) {
    static uint32_t lastClock = 0;
    uint32_t diff;
    uint32_t clock;
    uint32_t expectedDelay;

    // Animations are disabled when speed is zero.
    if (pots.var == 0) {
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

// Read a value from an analog pin as a byte value between 0-255.
uint8_t adc8(uint8_t pin) {
    uint16_t analog = analogRead(pin);
    return map(analog, 1023, 0, 0, 255);
}

// Return the switch position from 0-9.
uint8_t switchPosition(uint8_t pin) {
    uint16_t analog = analogRead(pin);
    // readings are 91, 128, 176, 235, 509, 605, 767, 695, 930, 958
    if (analog < 120)   return 0;
    if (analog < 170)   return 1;
    if (analog < 230)   return 2;
    if (analog < 500)   return 3;
    if (analog < 600)   return 4;
    if (analog < 690)   return 5;
    if (analog < 760)   return 7;
    if (analog < 920)   return 6;
    if (analog < 950)   return 8;
    return 9;
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
    fill_solid(&leds[0], NUM_LEDS, CHSV(pots.hue, pots.sat, pots.val));
}

// Fill all LEDs with a solid color on the temperature scale. The color
// breathes in and out to another temperature, defined by the extra parameter.
void modeTemperature() {
    uint8_t wave;
    uint8_t hue;

    wave = sin8(animation(230));
    hue = map8(wave, pots.hue, max(pots.var, pots.hue));

    fill_solid(&leds[0], NUM_LEDS, HeatColor(hue));
    FastLED.setBrightness(pots.val);
}

// Draw a linear gradient between two colors. The additional parameter defines
// the destination hue. The same saturation and value applies to both colors.
void modeGradient() {
    fill_gradient(
        &leds[0],
        0,          CHSV(pots.hue, pots.sat, pots.val),
        NUM_LEDS-1, CHSV(pots.var, pots.sat, pots.val)
    );
}

// Animate the rainbow. The hue parameter moves the rainbow back and forth,
// while the additional parameter regulates the animation speed.
void modeRainbow() {
    uint8_t hue;
    for (uint8_t led = 0; led < NUM_LEDS; led++) {
        hue = ledAngle(led) + animation(pots.var) + pots.hue;
        leds[led] = CHSV(hue, pots.sat, pots.val);
    }
}

// Fill all LEDs with a single color. Animate the intensity as a sine wave.
void modeBreathe() {
    uint8_t start;
    uint8_t sine;
    uint8_t value;

    sine = cubicwave8(animation(pots.var));
    start = pots.val - 100;
    if (start > pots.val) {
        start = 0;
    }
    value = map8(sine, start, pots.val);

#ifdef DEBUG_ANIMATION
    char buf[128];
    sprintf(buf, "sine:%4d  start:%4d  pot:%4d  value:%4d", sine, start, pots.val, value);
    Serial.println(buf);
    delay(20);
#endif

    fill_solid(&leds[0], NUM_LEDS, CHSV(pots.hue, pots.sat, value));
}

// Fill all LEDs with a single color, animating between two hues in a cubic
// ease in-out wave resembling a sine wave. The speed is calculated
// automatically based on distance on the color wheel.
void modeBreatheGradient() {
    uint8_t angle;
    uint8_t hue;
    uint8_t distance;
    uint8_t speed;

    distance = pots.var - pots.hue;
    speed = 255 - map8(distance, 15, 130);

    angle = cubicwave8(animation(speed));
    hue = map8(angle, pots.hue, pots.var);

    fill_solid(&leds[0], NUM_LEDS, CHSV(hue, pots.sat, pots.val));
}

// 1-N colored dots, weaving in and out of sync with each other. The variable
// adjusts the number of dots between 1 and half the LED strip.
//
// This function is adapted from https://github.com/atuline/FastLED-Demos.
void modeJuggle() {
    uint8_t led;
    uint8_t hue = pots.hue;
    uint8_t dots = map8(pots.var, 1, NUM_LEDS / 2);
    uint8_t step = 256 / dots;

    fadeToBlackBy(leds, NUM_LEDS, 20);

    for(uint8_t dot = 0; dot < dots; dot++) {
        led = beatsin16(dot+6, 0, NUM_LEDS-1);
        leds[led] |= CHSV(hue, pots.sat, pots.val);
        hue += step;
    }
}

// Animate each LED's intensity according to a cubic ease in-out function.
void modeEase() {
    uint8_t angle;
    uint8_t value;
    uint8_t an = animation(pots.var);
    for (uint8_t led = 0; led < NUM_LEDS; led++) {
        angle = map(led, 0, NUM_LEDS-1, 0, 255);
        value = ease8InOutCubic(sin8(angle + an));
        value = map(value, 0, 255, 0, pots.val);
        leds[led] = CHSV(pots.hue, pots.sat, value);
    }
}

// Animate a rainbow easing in and out. The hue parameter decides how far the
// rainbow will stretch on the color wheel, while the variable determines the speed.
void modeRainbowTrain() {
    uint8_t hue = 0;
    uint8_t angle;
    uint8_t value;
    uint8_t an = animation(pots.var);
    for (uint8_t led = 0; led < NUM_LEDS; led++) {
        angle = map(led, 0, NUM_LEDS-1, 0, 255);
        value = ease8InOutCubic(sin8(angle + an));
        hue = map8(value, 0, pots.hue);
        value = map8(value, 0, pots.val);
        leds[led] = CHSV(hue, pots.sat, value);
    }
}

// Read all potentiometers, and run one iteration of the active mode.
void loop() {
#ifdef DEBUG_POTS
    debugPots();
    delay(100);
#endif

    pots.mode = switchPosition(PIN_POT_SWITCH);
    pots.var  = adc8(PIN_POT_TOP);
    pots.hue  = adc8(PIN_POT_LEFT);
    pots.sat  = adc8(PIN_POT_CENTER);
    pots.val  = adc8(PIN_POT_RIGHT);

    modes[pots.mode]();
    FastLED.show();
}
