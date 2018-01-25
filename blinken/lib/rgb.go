package lib

import (
	"math"

	colorful "github.com/lucasb-eyer/go-colorful"
)

const x = math.Pi / 180

// RGBA returns a color encoded as a 32-bit unsigned integer in ARGB order.
func RGBA(c colorful.Color) uint32 {
	r, g, b := c.RGB255()
	return (0xff << 24) | (uint32(r) & 0xff << 16) | (uint32(g) & 0xff << 8) | (uint32(b) & 0xff)
}

// LinearRGB is an easier way to invoke LinearRGB, using ints instead of floats.
func LinearRGB(r, g, b uint8) colorful.Color {
	return colorful.Color{
		float64(r) / 255.0,
		float64(g) / 255.0,
		float64(b) / 255.0,
	}
}

// Rad converts degrees to radians.
func Rad(d float64) float64 {
	return d * x
}

// MiredToKelvin converts a Mired color to Kelvin degrees.
// Home Assistant sends values between 156-500, which correspond to a range of
// approximately 2000-6500 kelvins.
func MiredToKelvin(mired uint16) uint16 {
	x := 1000000 / int(mired)
	return uint16(x)
}

func ColorTemperature(kelvin uint16, luminance float64) colorful.Color {
	var coords xy
	var ok bool
	baseTemperature := int(math.Floor(float64(kelvin)/100) * 100)
	if coords, ok = colorTemperature[baseTemperature]; !ok {
		coords = colorTemperature[6500]
	}
	return colorful.Xyy(coords[0], coords[1], luminance)
}
