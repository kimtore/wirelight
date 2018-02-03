package effect

import (
	"time"

	"github.com/ambientsound/wirelight/blinken/ledclient"
	colorful "github.com/lucasb-eyer/go-colorful"
)

var Effects = make(map[string]Effect, 0)

type Parameters struct {
	Name   string
	Color  colorful.Color
	Adjust float64
}

type Effect interface {
	Draw(*ledclient.Canvas, Parameters)
	Delay() time.Duration
}

// FillFunc executes a callback function for every LED in the canvas. The
// callback function must return the new LED color. Arguments to the callback
// function is the physical LED coordinates and the existing color.
func FillFunc(canvas *ledclient.Canvas, f func(x, y int, c colorful.Color) colorful.Color) {
	width, height := canvas.Size()
	for x := 0; x < width; x++ {
		for y := 0; y < height; y++ {
			c := canvas.At(x, y)
			col := f(x, y, c)
			canvas.Set(x, y, col)
		}
	}
}

// Fill fills the entire color with one color.
func Fill(canvas *ledclient.Canvas, col colorful.Color) {
	FillFunc(canvas, func(x, y int, c colorful.Color) colorful.Color {
		return col
	})
}

// Run runs an effect forever.
func Run(canvas *ledclient.Canvas, p Parameters, terminate chan int) {
	timer := time.NewTimer(0)

	e := Effects[p.Name]

	reset := func() {
		e.Draw(canvas, p)
		timer = time.NewTimer(e.Delay())
	}

	for {
		select {
		case <-terminate:
			return
		case <-timer.C:
			reset()
		}
	}
}
