package effect

import (
	"math/rand"
	"time"

	colorful "github.com/lucasb-eyer/go-colorful"
)

type northernLights struct{}

func init() {
	Effects["northernLights"] = northernLights{}
}

func (e northernLights) Delay() time.Duration {
	return 10 * time.Millisecond
}

func (e northernLights) Draw(p Parameters) {
	h, c, l := p.Color.Hcl()
	def := colorful.Hcl(h, c, l)
	FillFunc(p.Canvas, func(x, y int, col colorful.Color) colorful.Color {
		if rand.Intn(100) != 0 {
			return def.BlendRgb(col, 0.98)
		}
		a := 180.0 * (1.0 / float64(rand.Intn(500)+1))
		return colorful.Hcl(h+a, c, rand.Float64()*l*2)
	})
}
