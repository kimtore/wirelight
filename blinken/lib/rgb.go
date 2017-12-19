package lib

import (
	"image/color"
	"math"

	colorful "github.com/lucasb-eyer/go-colorful"
)

const x = math.Pi / 180

// RGBA returns a color encoded as a 32-bit unsigned integer in ARGB order.
func RGBA(c color.Color) uint32 {
	r, g, b, a := c.RGBA()
	return (a & 0xff << 24) | (r & 0xff << 16) | (g & 0xff << 8) | (b & 0xff)
}

// MakeColor converts a Color to the corresponding Colorful type.
// This is a workaround for https://github.com/lucasb-eyer/go-colorful/issues/21.
func MakeColor(c color.Color) colorful.Color {
	r, g, b, _ := c.RGBA()
	return colorful.Color{
		float64(r) / 65535.0,
		float64(g) / 65535.0,
		float64(b) / 65535.0,
	}
}

// Rad converts degrees to radians.
func Rad(d float64) float64 {
	return d * x
}
