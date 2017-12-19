package lib_test

import (
	"image/color"
	"testing"

	"github.com/ambientsound/wirelight/blinken/lib"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
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

// Test that the color conversion function works as expected.
func TestMakeColor(t *testing.T) {
	for _, test := range rgbTests {
		c := lib.MakeColor(test.c)
		rt, gt, bt, _ := test.c.RGBA()
		rc, gc, bc, _ := c.RGBA()
		assert.Equal(t, rt, rc)
		assert.Equal(t, gt, gc)
		assert.Equal(t, bt, bc)
	}
}

// Test that the color conversion function works as expected.
func TestMakeColor2(t *testing.T) {
	var r, g, b, a uint8
	for r = 0; r < 255; r++ {
		for g = 0; g < 255; g++ {
			for b = 0; b < 255; b++ {
				ct := color.RGBA{r, g, b, a}
				cc := lib.MakeColor(ct)
				rt, gt, bt, _ := ct.RGBA()
				rc, gc, bc, _ := cc.RGBA()
				require.Equal(t, rt, rc)
				require.Equal(t, gt, gc)
				require.Equal(t, bt, bc)
			}
		}
	}
}
