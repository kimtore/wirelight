package mqttcolor_test

import (
	"testing"

	"github.com/ambientsound/wirelight/blinken/mqttcolor"
	colorful "github.com/lucasb-eyer/go-colorful"
	"github.com/stretchr/testify/assert"
)

var (
	COLOR colorful.Color = colorful.Hsl(100, 0.5, 0.5)
)

var mqttcolorTests = []struct {
	command  string
	current  mqttcolor.State
	expected mqttcolor.State
	err      bool
}{
	{
		"100,50,50",
		mqttcolor.State{},
		mqttcolor.State{
			Color: COLOR,
		},
		false,
	},
	{
		"ON",
		mqttcolor.State{
			Color: COLOR,
			On:    false,
		},
		mqttcolor.State{
			Color: COLOR,
			On:    true,
		},
		false,
	},
	{
		"OFF",
		mqttcolor.State{
			Color: COLOR,
			On:    true,
		},
		mqttcolor.State{
			Color: COLOR,
			On:    false,
		},
		false,
	},
	{
		"invalid",
		mqttcolor.State{},
		mqttcolor.State{},
		true,
	},
	{
		"1,-1",
		mqttcolor.State{},
		mqttcolor.State{},
		true,
	},
}

func TestMqttcolor(t *testing.T) {
	for _, test := range mqttcolorTests {
		result, err := test.current.Update(test.command)
		assert.Equal(t, test.expected, result)
		if test.err {
			assert.NotNil(t, err)
		} else {
			assert.Nil(t, err)
		}
	}
}
