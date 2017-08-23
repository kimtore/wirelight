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
* *Violet* changes lit segment size.

### Mode

* Solid color: displays a single color across all LEDs.
* Solid rainbow: cycle through the color wheel, showing a single color.
* Rainbow: color the LEDs with red to purple.
* Temperature: white leds, tinted red or blue according to temperature.

### Hue

Selects the color from the rainbow color wheel. This parameter has a value
between 0-255.

### Saturation

Sets the color saturation of all LEDs. This parameter has a value between
0-255, where zero equals white or no color.

### Value

Sets the brightness of all luminated LEDs. This parameter has a value between
0-255, where zero is completely dark.

### Speed

Controls the speed of the animation. The speed is increased by one each dial,
with the formula being:

    wait_time = 1 second / speed

If speed is zero, animations are disabled.
