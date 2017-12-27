package effect

import (
	"image"
	"time"

	"github.com/ambientsound/wirelight/blinken/lib"
	colorful "github.com/lucasb-eyer/go-colorful"
)

var Effects = make(map[string]Effect, 0)

type Palette map[string]colorful.Color

type Effect struct {
	Name       string              // Human-readable name.
	Function   func(Effect) Effect // Function that runs the effect.
	Palette    Palette             // Collection of colors available to the effect.
	Parameters map[string]float64  // Collection of parameters available to the effect.
	Terminate  chan int            // Send an integer to this channel to stop the effect.
	Canvas     *image.RGBA         // Canvas to draw the effect on.
	Delay      time.Duration       // Delay between each iteration.
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

// Run runs an effect forever.
func Run(e Effect, terminate chan int, canvas *image.RGBA) {
	e.Terminate = terminate
	e.Canvas = canvas

	timer := time.NewTimer(0)
	for {
		select {
		case <-e.Terminate:
			return
		case <-timer.C:
			e = e.Function(e)
			timer = time.NewTimer(e.Delay)
		}
	}
}
