package ledclient_test

import (
	"testing"

	"github.com/ambientsound/wirelight/blinken/ledclient"
	"github.com/stretchr/testify/assert"
)

var pixelIndexTests = []struct {
	x     int
	y     int
	index int
}{
	{0, 0, 0},
	{0, 1, 5},
	{4, 2, 14},
	{4, 9, 49},
}

func TestPixelIndex(t *testing.T) {
	canvas := ledclient.NewCanvas(5, 10)
	for _, test := range pixelIndexTests {
		i := canvas.PixelIndex(test.x, test.y)
		assert.Equal(t, test.index, i)
	}
}
