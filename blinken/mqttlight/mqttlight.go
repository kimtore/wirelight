// package mqttlight provides support for the Home-Assistant light.mqtt_json component.
package mqttlight

import "encoding/json"

// Which kind of JSON message was sent.
type CommandType int

const (
	Unknown CommandType = iota
	Brightness
	Effect
	RGB
	State
	Temperature
	White
)

// Command holds the message from the Home-Assistant
type Command struct {
	Brightness int
	Color_temp int
	Color      struct {
		R int
		G int
		B int
	}
	Effect      string
	State       string
	Transition  int
	White_value int
}

// Unmarshal converts a JSON payload into a command structure.
func Unmarshal(cmd []byte) (Command, error) {
	c := Command{}
	err := json.Unmarshal(cmd, &c)
	return c, err
}

// Type returns the update command type, based on which fields were set during
// JSON unmarshalling.
func (c Command) Type() CommandType {
	if c.Brightness > 0 {
		return Brightness
	}
	// TODO: effect
	if c.Color_temp > 0 {
		return Temperature
	}
	if c.Color.R > 0 || c.Color.G > 0 || c.Color.B > 0 {
		return RGB
	}
	if c.White_value > 0 {
		return White
	}
	if len(c.State) > 0 {
		return State
	}
	return Unknown
}
