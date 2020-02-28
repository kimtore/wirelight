# servo

Servo is a middleware between ESPHome NeoPixel lights and OpenHAB. It translates between OpenHAB's `R,G,B`
and ESPHome's JSON `{"color":{"r":R,"g":G,"b":B}}` via a MQTT integration.

Add a generic MQTT device in OpenHAB, give it a RGB channel, and prefix the topic the ESPHome lives on with `servo/`.
For example, command topic for ESPHome light is `laito/light/neopixels/command`, use `servo/laito/light/neopixels/command`.

Configuration should work out of the box, provided your lights live at `+/light/neopixels/+`.

```
# MQTT configuration variables
export SERVO_USERNAME SERVO_SERVER SERVO_PASSWORD
```
