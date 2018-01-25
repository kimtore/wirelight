package ledclient

import colorful "github.com/lucasb-eyer/go-colorful"

// Canvas is an array of LEDs, powered by colorful colors.
type Canvas struct {
	pixels []colorful.Color
	width  int
	height int
}

func NewCanvas(width, height int) *Canvas {
	return &Canvas{
		pixels: make([]colorful.Color, width*height),
		width:  width,
		height: height,
	}
}

func (c *Canvas) pixelIndex(x, y int) int {
	return c.height*y + x
}

func (c *Canvas) Set(x, y int, col colorful.Color) {
	c.pixels[c.pixelIndex(x, y)] = col
}

func (c *Canvas) At(x, y int) colorful.Color {
	return c.pixels[c.pixelIndex(x, y)]
}

func (c *Canvas) Size() (int, int) {
	return c.width, c.height
}
