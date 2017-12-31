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
		Delay:    18000 * time.Microsecond,
		Palette: Palette{
			"default": colorful.Hcl(0, 0, 0),
		},
	}
}

func wave(e Effect) Effect {
	h, s, v := e.Palette["default"].Clamped().Hsv()
	bounds := e.Canvas.Bounds()
	xmax := float64(bounds.Max.X)
	xstep := 180.0 / xmax

	FillFunc(e.Canvas, func(x, y int, col colorful.Color) colorful.Color {
		lumAngle := waveSine + (float64(x) * xstep)
		sin := math.Abs(math.Sin(lib.Rad(lumAngle)))
		val := v - (sin * 4)
		return colorful.Hsv(h, s, val)
	})

	waveSine += 0.1
	if waveSine >= 180.0 {
		waveSine = -waveSine
	}

	return e
}
