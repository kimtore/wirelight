package lib_test

import (
	"image/color"
	"testing"

	"github.com/ambientsound/wirelight/blinken/lib"
	"github.com/stretchr/testify/assert"
)

var rgbTests = []struct {
	c color.Color
	i uint32
}{
	{color.RGBA{255, 0, 0, 0}, 0x00ff0000},
	{color.RGBA{0, 255, 0, 0}, 0x0000ff00},
	{color.RGBA{0, 0, 255, 0}, 0x000000ff},
	{color.RGBA{255, 255, 0, 0}, 0x00ffff00},
	{color.RGBA{0, 255, 255, 0}, 0x0000ffff},
	{color.RGBA{255, 255, 255, 0}, 0x00ffffff},
	{color.RGBA{0, 255, 255, 255}, 0xff00ffff},
	{color.RGBA{255, 255, 255, 255}, 0xffffffff},
}

func TestRGB(t *testing.T) {
	for _, test := range rgbTests {
		i := lib.RGBA(test.c)
		assert.Equal(t, test.i, i)
	}
}
