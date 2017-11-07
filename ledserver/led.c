/*
 * LEDServer by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
 *
 * This program drives LEDs from a Raspberry Pi Zero W using the rpi_ws281x
 * library and Google Protobuf messages.
 */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <signal.h>
#include <stdarg.h>

#include "rpi_ws281x/version.h"
#include "rpi_ws281x/ws2811.h"

#include "led.h"

#define DMA                     5
#define GPIO_PIN                18
#define LED_COUNT               60
#define MAX_BRIGHTNESS          80
#define STRIP_TYPE              WS2812_STRIP
#define TARGET_FREQ             WS2811_TARGET_FREQ

static ws2811_t ledstring;

static uint8_t running = 1;

void ledstrip_clear(void) {
    for (int i = 0; i < LED_COUNT; i++) {
        ledstring.channel[0].leds[i] = 0;
    }
}

int ledstrip_init() {
    ledstring.freq = TARGET_FREQ,
    ledstring.dmanum = DMA,
    ledstring.channel[0].gpionum = GPIO_PIN;
    ledstring.channel[0].count = LED_COUNT;
    ledstring.channel[0].invert = 0;
    ledstring.channel[0].brightness = MAX_BRIGHTNESS;
    ledstring.channel[0].strip_type = STRIP_TYPE;
    ledstring.channel[1].gpionum = 0;
    ledstring.channel[1].count = 0;
    ledstring.channel[1].invert = 0;
    ledstring.channel[1].brightness = 0;

    ws2811_return_t ret;
    if ((ret = ws2811_init(&ledstring)) != WS2811_SUCCESS) {
        fprintf(stderr, "ws2811_init failed: %s\n", ws2811_get_return_t_str(ret));
    }

    return ret;
}

void ledstrip_finish() {
    ledstrip_clear();
    ws2811_render(&ledstring);
    ws2811_fini(&ledstring);
}

int ledstrip_render() {
    ws2811_return_t ret;
    if ((ret = ws2811_render(&ledstring)) != WS2811_SUCCESS) {
        fprintf(stderr, "ws2811_render failed: %s\n", ws2811_get_return_t_str(ret));
    }
    return ret;
}

void ledstrip_assign(uint32_t led, uint32_t value) {
    if (led >= LED_COUNT) {
        return;
    }
    ledstring.channel[0].leds[led] = value;
}

static void ctrl_c_handler(int signum)
{
    (void)(signum);
    running = 0;
}

static void setup_handlers(void)
{
    struct sigaction sa;
    sa.sa_handler = ctrl_c_handler;

    sigaction(SIGINT, &sa, NULL);
    sigaction(SIGTERM, &sa, NULL);
}

int main(int argc, char *argv[])
{
    ledstrip_init();

    setup_handlers();

    while (running)
    {
        ledstrip_assign(29, 127);
        ledstrip_assign(30, 255);
        ledstrip_render();

        // 15 frames /sec
        usleep(1000000 / 15);
    }

    ledstrip_clear();
    ledstrip_finish();

    printf ("\n");

    return 0;
}
