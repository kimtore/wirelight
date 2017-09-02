#include <Adafruit_NeoPixel.h>

#define OUTPUT_PIN 5
#define MAX_MODES 10
#define PULSE_INTENSITY_MIN 50

Adafruit_NeoPixel strip = Adafruit_NeoPixel(30, OUTPUT_PIN, NEO_GRB + NEO_KHZ800);

void mode_off();
void mode_solid_color();
void mode_pulsing_solid_color();
void mode_rainbow();
void mode_rainbow_cycle();

void (*modes[MAX_MODES])() = {
  mode_off,
  mode_solid_color,
  mode_pulsing_solid_color,
  mode_rainbow,
  mode_rainbow_cycle,
  mode_off,
  mode_off,
  mode_off,
  mode_off,
  mode_off
};

void setup() {
  Serial.begin(9600);
  pinMode(OUTPUT_PIN, OUTPUT);
  analogReference(DEFAULT);
  strip.begin();
  strip.show(); // Initialize all pixels to 'off'
}

void debug_all() {
  byte i;
  uint16_t knobs[5];
  char buf[64];
  for (i=0; i<5; i++) {
    knobs[i] = analogRead(i+1);
    sprintf(buf, "Knob %d = %d\n", i, knobs[i]);
    Serial.write(buf);
  }
  delay(1000);
}

byte read_mode() {
  uint16_t value = analogRead(5);
  if (value < 100) {
    return 0;
  } else if (value < 150) {
    return 1;
  } else if (value < 200) {
    return 2;
  } else if (value < 250) {
    return 3;
  } else if (value < 550) {
    return 4;
  } else if (value < 650) {
    return 5;
  } else if (value < 750) {
    return 6;
  } else if (value < 850) {
    return 7;
  } else if (value < 950) {
    return 8;
  } else {
    return 9;
  }
}

byte adc_to_8bit(uint16_t adc) {
  adc = 1023 - adc;
  adc <<= 6;
  adc >>= 8;
  return adc;
}

byte read_byte(byte input) {
  return adc_to_8bit(analogRead(input));
}

byte read_delay(byte input, byte modifier) {
  return (255 - read_byte(input)) / modifier;
}

void read_rgb(byte * rgb) {
  byte i;
  for (i = 0; i < 3; i++) {
    rgb[i] = read_byte(i+2);
  }
}

void intensify_rgb(byte * rgb, byte intensity) {
  byte max_;
  double value;
  byte i;
  max_ = max(rgb[0], max(rgb[1], rgb[2]));
  if (max_ == 0) {
    for (i = 0; i < 3; i++) {
      rgb[i]++;
    }
    max_ = 1;
  }
  for (i = 0; i < 3; i++) {
    value = rgb[i];
    value = (value / max_) * intensity;
    rgb[i] = value;
  }
}

// Input a value 0 to 255 to get a color value.
// The colours are a transition r - g - b - back to r.
uint32_t Wheel(byte WheelPos) {
  WheelPos = 255 - WheelPos;
  if(WheelPos < 85) {
    return strip.Color(255 - WheelPos * 3, 0, WheelPos * 3);
  }
  if(WheelPos < 170) {
    WheelPos -= 85;
    return strip.Color(0, WheelPos * 3, 255 - WheelPos * 3);
  }
  WheelPos -= 170;
  return strip.Color(WheelPos * 3, 255 - WheelPos * 3, 0);
}

void mode_off() {
  uint16_t i;
  for (i = 0; i < strip.numPixels(); i++) {
    strip.setPixelColor(i, 0);
  }
}

void mode_solid_color() {
  byte rgb[3];
  byte i;
  
  read_rgb(rgb);
  intensify_rgb(rgb, read_byte(1));
  
  for(i = 0; i < strip.numPixels(); i++) {
    uint32_t c = strip.Color(rgb[0], rgb[1], rgb[2]);
    strip.setPixelColor(i, c);
  }
}

void mode_pulsing_solid_color() {
  byte rgb[3];
  byte i;
  static int8_t delta = 1;
  static byte intensity = PULSE_INTENSITY_MIN;

  delay(read_delay(1, 5));

  intensity += delta;
  if (intensity < PULSE_INTENSITY_MIN) {
    delta *= -1;
    intensity += delta;
  }

  read_rgb(rgb);
  intensify_rgb(rgb, intensity);
  
  for(i = 0; i < strip.numPixels(); i++) {
    uint32_t c = strip.Color(rgb[0], rgb[1], rgb[2]);
    strip.setPixelColor(i, c);
  }
}

void mode_rainbow() {
  byte i;
  static byte wheel_position = 0;

  delay(read_delay(1, 3));
  
  for(i = 0; i < strip.numPixels(); i++) {
    strip.setPixelColor(i, Wheel((i + wheel_position) & 255));
  }

  ++wheel_position;
}

void mode_rainbow_cycle() {
  byte i;
  static byte wheel_position = 0;

  delay(read_delay(1, 3));
  
  for(i = 0; i < strip.numPixels(); i++) {
    strip.setPixelColor(i, Wheel(((i * 256 / strip.numPixels()) + wheel_position) & 255));
  }

  ++wheel_position;
}

// Slightly different, this makes the rainbow equally distributed throughout
void rainbowCycle(uint8_t wait) {
  uint16_t i, j;

  for(j=0; j<256*5; j++) { // 5 cycles of all colors on wheel
    for(i=0; i< strip.numPixels(); i++) {
      strip.setPixelColor(i, Wheel(((i * 256 / strip.numPixels()) + j) & 255));
    }
    strip.show();
    delay(wait);
  }
}

void loop() {
  byte mode;
  //debug_all();
  mode = read_mode();
  modes[mode]();
  strip.show();
}

