package effect

import (
	"math/rand"
	"time"

	colorful "github.com/lucasb-eyer/go-colorful"
)

func init() {
	Effects["northernLights"] = Effect{
		Name:     "Northern lights",
		Function: northernLights,
		Delay:    10 * time.Millisecond,
		Palette: Palette{
			"default": colorful.Hcl(0, 0, 0),
		},
	}
}

func northernLights(e Effect) Effect {
	h, c, l := e.Palette["default"].Hcl()
	def := colorful.Hcl(h, c, l)
	FillFunc(e.Canvas, func(x, y int, col colorful.Color) colorful.Color {
		if rand.Intn(100) != 0 {
			return def.BlendRgb(col, 0.98)
		}
		a := 180.0 * (1.0 / float64(rand.Intn(500)+1))
		return colorful.Hcl(h+a, c, rand.Float64()*l*2)
	})
	return e
}
