package mqttcolor_test

import (
	"testing"

	"github.com/ambientsound/wirelight/blinken/mqttcolor"
	colorful "github.com/lucasb-eyer/go-colorful"
	"github.com/stretchr/testify/assert"
)

var (
	COLOR colorful.Color = colorful.Hsl(100, 0.5, 0.125)
)

var mqttcolorTests = []struct {
	command       string
	current       mqttcolor.State
	expected      mqttcolor.State
	expectedColor colorful.Color
	err           bool
}{
	{
		"100,50,50",
		mqttcolor.State{},
		mqttcolor.State{
			On:    true,
			Color: COLOR,
		},
		COLOR,
		false,
	},
	{
		"ON",
		mqttcolor.State{
			On:    false,
			Color: COLOR,
		},
		mqttcolor.State{
			On:    true,
			Color: COLOR,
		},
		COLOR,
		false,
	},
	{
		"OFF",
		mqttcolor.State{
			On:    true,
			Color: COLOR,
		},
		mqttcolor.State{
			On:    false,
			Color: COLOR,
		},
		colorful.Color{},
		false,
	},
	{
		"invalid",
		mqttcolor.State{},
		mqttcolor.State{},
		colorful.Color{},
		true,
	},
	{
		"1,-1",
		mqttcolor.State{},
		mqttcolor.State{},
		colorful.Color{},
		true,
	},
}

func TestMqttcolor(t *testing.T) {
	for i, test := range mqttcolorTests {
		t.Logf("Running test %d\n", i+1)
		result, err := test.current.Update(test.command)
		assert.Equal(t, test.expected, result)
		sc := result.SwitchedColor()
		assert.Equal(t, test.expectedColor, sc)
		if test.err {
			assert.NotNil(t, err)
		} else {
			assert.Nil(t, err)
		}
	}
}
