// package mqttlight provides support for the Home-Assistant light.mqtt_json component.
package mqttlight

import (
	"encoding/json"

	"github.com/ambientsound/wirelight/blinken/lib"
	colorful "github.com/lucasb-eyer/go-colorful"
)

// Which kind of JSON message was sent.
type StateType int

const (
	Unknown StateType = iota
	BrightnessChanged
	EffectChanged
	RGBChanged
	StateChanged
	TemperatureChanged
	WhiteValueChanged
)

// State holds the message from the Home-Assistant
type State struct {
	Brightness uint8
	Color_temp uint16
	Color      struct {
		R uint8
		G uint8
		B uint8
	}
	Effect      string
	State       string
	Transition  int
	White_value int
}

// Unmarshal converts a JSON payload into a command structure.
func Unmarshal(cmd []byte) (State, error) {
	c := State{}
	err := json.Unmarshal(cmd, &c)
	return c, err
}

// Type returns the update command type, based on which fields were set during
// JSON unmarshalling.
func (c State) Type() StateType {
	if c.Brightness > 0 {
		return BrightnessChanged
	}
	// TODO: effect
	if c.Color_temp > 0 {
		return TemperatureChanged
	}
	if c.Color.R > 0 || c.Color.G > 0 || c.Color.B > 0 {
		return RGBChanged
	}
	if c.White_value > 0 {
		return WhiteValueChanged
	}
	if len(c.State) > 0 {
		return StateChanged
	}
	return Unknown
}

func (c State) On() bool {
	return c.State == "ON"

}

func (c State) TransformColor(existing colorful.Color) colorful.Color {
	switch c.Type() {
	case RGBChanged:
		return lib.LinearRGB(c.Color.R, c.Color.G, c.Color.B)
	case TemperatureChanged:
		kelvin := lib.MiredToKelvin(c.Color_temp)
		return lib.ColorTemperature(kelvin, 1.0)
	}
	return existing
}
