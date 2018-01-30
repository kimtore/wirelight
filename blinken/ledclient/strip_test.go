package ledclient_test

import (
	"testing"

	"github.com/ambientsound/wirelight/blinken/ledclient"
	"github.com/stretchr/testify/assert"
)

var stripIndexTests = []struct {
	x     int
	y     int
	index uint32
}{
	{0, 0, 0},
	{1, 0, 1},
	{2, 0, 2},
	{3, 0, 3},
	{0, 1, 7},
	{1, 1, 6},
	{2, 1, 5},
	{3, 1, 4},
	{0, 2, 8},
	{1, 2, 9},
	{2, 2, 10},
	{3, 2, 11},
}

func TestStripIndex(t *testing.T) {
	strip := ledclient.NewStrip(nil, 4, 3, 0)
	for _, test := range stripIndexTests {
		i := strip.Index(test.x, test.y)
		assert.Equal(t, test.index, i)
	}
}
