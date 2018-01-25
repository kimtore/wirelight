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
		Delay:    400 * time.Microsecond,
		Palette: Palette{
			"default": colorful.Hcl(0, 0, 0),
		},
	}
}

func wave(e Effect) Effect {
	h, s, v := e.Palette["default"].Hsv()
	width, _ := e.Canvas.Size()
	xstep := 180.0 / float64(width) // wave length equals one strip length

	FillFunc(e.Canvas, func(x, y int, col colorful.Color) colorful.Color {
		lumAngle := waveSine + (float64(x) * xstep)
		sin := (1 + math.Sin(lib.Rad(lumAngle))) / 4
		val := v + sin
		return colorful.Hsv(h, s, val)
	})

	waveSine += 0.1
	if waveSine >= 180.0 {
		waveSine = -waveSine
	}

	return e
}
