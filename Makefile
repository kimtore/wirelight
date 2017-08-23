# Arduino Make file. Refer to https://github.com/sudar/Arduino-Makefile

ARDUINO_LIBS = Wire FastLED Encoder
BOARD_TAG = nano
BOARD_SUB = atmega328
TARGET = wirelight
include ~/src/Arduino-Makefile/Arduino.mk
