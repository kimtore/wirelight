package effect

import (
	"math"
	"time"

	"github.com/ambientsound/wirelight/blinken/lib"
	colorful "github.com/lucasb-eyer/go-colorful"
)

var waveSine float64 = 0.0

func init() {
	Effects["wave"] = Effect{
		Name:     "Wave",
		Function: wave,
		Delay:    180 * time.Microsecond,
		Palette: Palette{
			"default": colorful.Hcl(0, 0, 0),
		},
	}
}

func wave(e Effect) Effect {
	h, c, l := e.Palette["default"].Hcl()
	bounds := e.Canvas.Bounds()
	xmax := float64(bounds.Max.X)
	xstep := 180.0 / xmax

	FillFunc(e.Canvas, func(x, y int, col colorful.Color) colorful.Color {
		lumAngle := waveSine + (float64(x) * xstep)
		sin := (1 + math.Sin(lib.Rad(lumAngle))) / 4
		val := l + sin
		return colorful.Hcl(h, c, val)
	})

	waveSine += 0.1
	if waveSine >= 180.0 {
		waveSine = -waveSine
	}

	return e
}
