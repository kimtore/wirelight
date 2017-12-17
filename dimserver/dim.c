/*
 * Dimserver by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
 *
 * This program drives a DAC which outputs 0-10V in order to dim the fluorescent tubes.
 *
 * The fluorescent tubes are dimmed with voltage values ranging from 1.0-10.0V.
 * The DAC requires an input and reference voltage of 11.22V in order to give
 * 10.00V output. The DAC is not linear, so 1.00V is output at a digital value
 * of 366, while 10.00V is output at a digital value of 4095.
 */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include <wiringPi.h>

#define DAC_BITS        12
#define MAX_VOLTAGE     10.0
#define VOLTAGE_STEP    (MAX_VOLTAGE/4096.0)

// GPIO pinouts, using BCM pin numbers
#define PIN_CLK         17
#define PIN_DIN         27
#define PIN_LOAD        22

// Initialize the WiringPI library and the GPIO pins.
void dim_init()
{
    wiringPiSetupGpio();
    pinMode(PIN_CLK, OUTPUT);
    pinMode(PIN_DIN, OUTPUT);
    pinMode(PIN_LOAD, OUTPUT);
    digitalWrite(PIN_CLK, LOW);
    digitalWrite(PIN_DIN, LOW);
    digitalWrite(PIN_LOAD, HIGH);
}

// Return a single bit of a number.
static inline int dim_shiftreg_bit(uint16_t value, int shift)
{
    return (value >> shift) & 0x1;
}

// Load the contents of the shift register into Vout.
static inline void dim_shiftreg_load()
{
    digitalWrite(PIN_LOAD, LOW);
    digitalWrite(PIN_LOAD, HIGH);
}

// Push a single bit into the shift register.
static inline void dim_shiftreg_push(int bit)
{
    digitalWrite(PIN_DIN, bit);
    digitalWrite(PIN_CLK, HIGH);
    digitalWrite(PIN_CLK, LOW);
}

// High level function that sets Vout to a value between 0.00-10.00V, using
// values between 0-4095.
void dim_shiftreg_set(uint16_t value)
{
    int shift = DAC_BITS;
    int bit;

    while (shift > 0) {
        --shift;
        bit = dim_shiftreg_bit(value, shift);
        dim_shiftreg_push(bit);
    }

    dim_shiftreg_load();
}

int main(int argc, char **argv)
{
    uint16_t value = 0;
    float volts = 0;

    dim_init();

    if (argc > 0) {
        value = atoi(argv[1]);
    }

    volts = value * VOLTAGE_STEP;
    printf("Setting value to %d, should correspond to %5.3fV\n", value, volts);
    dim_shiftreg_set(value);

    return 0;
}
