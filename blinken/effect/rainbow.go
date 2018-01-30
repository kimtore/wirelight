package effect

import (
	"time"

	colorful "github.com/lucasb-eyer/go-colorful"
)

var rainbowSine float64 = 0.0

func init() {
	Effects["rainbow"] = Effect{
		Name:     "Rainbow",
		Function: rainbow,
		Delay:    4000 * time.Microsecond,
		Palette: Palette{
			"default": colorful.Hcl(0, 0, 0),
		},
	}
}

func addUp(f, delta, max float64) float64 {
	f += delta
	for f >= max {
		f -= max
	}
	return f
}

func rainbow(e Effect) Effect {
	h, s, v := e.Palette["default"].Hsv()
	width, _ := e.Canvas.Size()
	h += waveSine
	hueStep := 140.0 / float64(width)

	FillFunc(e.Canvas, func(x, y int, col colorful.Color) colorful.Color {
		hue := addUp(h, float64(x)*hueStep, 360.0)
		return colorful.Hsv(hue, s, v)
	})

	waveSine += 0.1
	if waveSine >= 360.0 {
		waveSine = 0
	}

	return e
}
