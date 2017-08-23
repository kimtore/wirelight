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

* *Red* changes mode
* *Orange* changes hue
* *Yellow* changes saturation
* *Green* changes value
* *Blue* changes speed
* *Indigo* ?
* *Violet* ?

Colors of the rainbow:

* red hsl(0,100%,50%)
* gold hsl(51,100%,50%)
* chartreuse hsl(103,100%,50%)
* spring green hsl(154,100%,50%)
* dodger blue hsl(206,100%,50%)
* saturated slate blue hsl(257,100%,50%)
* deep pink - violet, arguably fuchsia hsl(309,100%,50%)

Alternate colors:

* red hsl(2,80%,50%)
* orange-red hsl(24,100%,50%)
* orange-yellow hsl(39,100%,50%)
* yellow-tinged green hsl(70,100%,50%)
* aquamarine hsl(158,100%,50%)
* royal blue hsl(230,70%,50%)
* medium purple hsl(252,100%,70%)

### Mode

* Solid color: displays a single color across all LEDs.
* Rainbow: color the LEDs with red to purple.

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
