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
	Angle  float64
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

func increment(source, incr, min, max float64) float64 {
	source += incr
	if source >= max {
		return min
	}
	return source
}

// Run runs an effect forever.
func Run(canvas *ledclient.Canvas, ch chan Parameters, terminate chan int) {
	var effect Effect
	var params Parameters

	timer := time.NewTimer(1000)

	reset := func() {
		effect.Draw(canvas, params)
		timer = time.NewTimer(effect.Delay())
	}

	for {
		select {
		case <-terminate:
			return
		case params = <-ch:
			effect = Effects[params.Name]
			reset()
		case <-timer.C:
			reset()
		}
		params.Angle = increment(params.Angle, 0.1, 0, 360)
	}
}
