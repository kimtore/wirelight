package lib_test

import (
	"testing"

	"github.com/ambientsound/wirelight/blinken/lib"
	colorful "github.com/lucasb-eyer/go-colorful"
	"github.com/stretchr/testify/assert"
)

var rgbTests = []struct {
	c colorful.Color
	i uint32
}{
	{lib.LinearRGB(0, 0, 0), 0xff000000},
	{lib.LinearRGB(255, 0, 0), 0xffff0000},
	{lib.LinearRGB(0, 255, 0), 0xff00ff00},
	{lib.LinearRGB(0, 0, 255), 0xff0000ff},
	{lib.LinearRGB(255, 255, 0), 0xffffff00},
	{lib.LinearRGB(0, 255, 255), 0xff00ffff},
	{lib.LinearRGB(127, 64, 32), 0xff7f4020},
}

func TestRGB(t *testing.T) {
	for _, test := range rgbTests {
		i := lib.RGBA(test.c)
		assert.Equal(t, test.i, i)
	}
}
