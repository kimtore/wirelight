package lib

import "image/color"

// RGBA returns a color encoded as a 32-bit unsigned integer in ARGB order.
func RGBA(c color.Color) uint32 {
	r, g, b, a := c.RGBA()
	return (a & 0xff << 24) | (r & 0xff << 16) | (g & 0xff << 8) | (b & 0xff)
}
