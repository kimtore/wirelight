# Wirelight

LED strips powered by Arduino.

## Building

Wirelight uses the [FastLED library](https://github.com/FastLED/FastLED).

## Operation

Wirelight is equipped with a multi-purpose rotary click button. The button
provides two functions:

* Button press: select parameter to adjust.
* Spin wheel left or right: adjust value of parameter.

When the button is pressed to change parameter, a solid color is shown in
maximum brightness across all LEDs. This effect disappears when the button is
de-pressed.

The solid colors are of the rainbow. Thus, only seven parameters are supported:

* *Red* changes mode.
* *Orange* changes hue (position on the color wheel).
* *Yellow* changes saturation (amount of color).
* *Green* changes value (brightness).
* *Blue* changes animation speed.
* *Indigo* changes start position.
* *Violet* changes lit LED count.

### Mode

- *Temperature*: white leds, which are tinted red, yellow or blue.
- *Solid color*: displays a single color across all LEDs.
- *Solid rainbow*: cycle through the color wheel if animations are enabled,
  showing a single color at a time.
- *Rainbow*: color the LEDs with red to purple.

### Hue

Selects the color from the rainbow color wheel.

### Saturation

Sets the color saturation of all LEDs. If saturation is set to zero, all LEDs
turn white.

### Value

Sets the brightness of all luminated LEDs.

### Animation speed

Controls the speed of the animation. The speed is increased by one each dial,
with the formula being:

    wait_time = 1 second / speed

If speed is zero, animations are disabled.

The animation time ranges between 4-1000ms.

### Start position

This setting moves lit LEDs around. It is only useful when the lit segment size
parameter is decreased so that only some LEDs are lit, and animations disabled.

Note that the start position will dynamically change when animations are
enabled.

### Lit LED count

Controls how many LEDs are lit up at any given time.
