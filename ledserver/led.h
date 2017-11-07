/*
 * LEDServer by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
 *
 * This program drives LEDs from a Raspberry Pi Zero W using the rpi_ws281x
 * library and Google Protobuf messages.
 */

int ledstrip_init();
int ledstrip_render();

void ledstrip_assign(uint32_t led, uint32_t value);
void ledstrip_clear(void);
void ledstrip_finish();
