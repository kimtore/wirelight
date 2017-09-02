// Use the FastLED library.
#include <FastLED.h>

// Serial DEBUG mode on/off
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
void modeOff();
void modeSolid();
void modeRainbow();
void modeRainbowCycle();
void modeCubic();
void (*modes[MAX_MODES])() = {
    &modeOff,
    &modeSolid,
    &modeRainbow,
    &modeRainbowCycle,
    &modeCubic,
    &modeOff,
    &modeOff,
    &modeOff,
    &modeOff,
    &modeOff
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
#ifdef DEBUG_POTS
    Serial.begin(115200);
#endif
}

#ifdef DEBUG_POTS
void debugPots() {
    uint8_t i;
    uint16_t analog;
    char buf[64];
    char *ptr = &buf[0];

    for (i=0; i<6; i++) {
        analog = analogRead(i);
        ptr += sprintf(ptr, "%4d, ", analog);
    }

    sprintf(ptr, "\0");
    Serial.println(buf);
}
#endif

// Convert analog voltage readings to a mode number.
uint8_t modeNumber(uint16_t analogValue) {
    if (analogValue < 100) {
        return 0;
    } else if (analogValue < 150) {
        return 1;
    } else if (analogValue < 200) {
        return 2;
    } else if (analogValue < 250) {
        return 3;
    } else if (analogValue < 550) {
        return 4;
    } else if (analogValue < 650) {
        return 5;
    } else if (analogValue < 750) {
        return 6;
    } else if (analogValue < 850) {
        return 7;
    } else if (analogValue < 950) {
        return 8;
    } else {
        return 9;
    }
}

// animate returns true if the animation should be stepped.
uint8_t animate() {
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

    expectedDelay = map(pots.var, 1, 255, 400, 5);
    if (diff >= expectedDelay) {
        lastClock = clock;
        return 1;
    }

    return 0;
}

// animation animates linearly, repeating the numbers 0-255.
uint8_t animation() {
    static uint8_t an = 0;
    an += animate();
    return an;
}

// animation animates linearly, repeating the numbers 0-255.
uint8_t animationCubic() {
    static uint8_t an = 0;
    static int8_t multiplier = 1;
    if (an == 255 && multiplier == 1) {
        multiplier = -1;
    } else if (an == 0 && multiplier == -1) {
        multiplier = 1;
    }
    an = an + (multiplier * animate());
    return an;
}

// Read a value from an analog pin as a byte value between 0-255.
uint8_t adc8(uint8_t pin) {
    uint16_t analog = analogRead(pin);
    return map(analog, 1023, 0, 0, 255);
}

// Switch off all LEDs.
void modeOff() {
    fill_solid(&leds[0], NUM_LEDS, CRGB::Black);
}

// Fill all LEDs with the same color. This mode does not have an additional parameter.
void modeSolid() {
    fill_solid(&leds[0], NUM_LEDS, CHSV(pots.hue, pots.sat, pots.val));
}

// Draw the rainbow. The additional parameter moves the rainbow back and forth.
void modeRainbow() {
    uint8_t hue;
    for (uint8_t led = 0; led < NUM_LEDS; led++) {
        hue = map(led, 0, NUM_LEDS, 0, 255);
        hue += pots.var;
        leds[led] = CHSV(hue, pots.sat, pots.val);
    }
}

// Draw the rainbow, and gradually move it according to the addition parameter,
// which regulates the speed.
void modeRainbowCycle() {
    uint8_t hue;
    for (uint8_t led = 0; led < NUM_LEDS; led++) {
        hue = map(led, 0, NUM_LEDS, 0, 255);
        hue += animation();
        leds[led] = CHSV(hue, pots.sat, pots.val);
    }
}

void modeCubic() {
    uint8_t cubic;
    uint8_t value;
    for (uint8_t led = 0; led < NUM_LEDS; led++) {
        cubic = animationCubic();
        value = map(cubic, 0, 255, min(70, pots.val), pots.val);
        leds[led] = CHSV(pots.hue, pots.sat, value);
    }
}

// Read all potentiometers, and run one iteration of the active mode.
void loop() {
#ifdef DEBUG_POTS
    debugPots();
    delay(100);
#endif

    pots.mode = modeNumber(analogRead(PIN_POT_SWITCH));
    pots.var  = adc8(PIN_POT_TOP);
    pots.hue  = adc8(PIN_POT_LEFT);
    pots.sat  = adc8(PIN_POT_CENTER);
    pots.val  = adc8(PIN_POT_RIGHT);

    modes[pots.mode]();
    FastLED.show();
}
