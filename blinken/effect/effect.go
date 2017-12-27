package effect

import (
	"image"

	"github.com/ambientsound/wirelight/blinken/lib"
	colorful "github.com/lucasb-eyer/go-colorful"
)

type Effect struct {
	Name       string
	Function   func(*image.RGBA)
	Palette    map[string]colorful.Color
	Parameters map[string]float64
}

// FillFunc executes a callback function for every LED in the canvas. The
// callback function must return the new LED color. Arguments to the callback
// function is the physical LED coordinates and the existing color.
func FillFunc(canvas *image.RGBA, f func(x, y int, c colorful.Color) colorful.Color) {
	b := canvas.Bounds()
	for x := b.Min.X; x < b.Max.X; x++ {
		for y := b.Min.Y; y < b.Max.Y; y++ {
			c := lib.MakeColor(canvas.At(x, y))
			col := f(x, y, c)
			canvas.Set(x, y, col.Clamped())
		}
	}
}

// Fill fills the entire color with one color.
func Fill(canvas *image.RGBA, col colorful.Color) {
	FillFunc(canvas, func(x, y int, c colorful.Color) colorful.Color {
		return col
	})
}
