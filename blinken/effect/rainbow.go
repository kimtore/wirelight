package effect

import (
	"time"

	"github.com/ambientsound/wirelight/blinken/ledclient"
	colorful "github.com/lucasb-eyer/go-colorful"
)

type rainbow struct{}

func init() {
	Effects["rainbow"] = rainbow{}
}

func (e rainbow) Delay() time.Duration {
	return 4000 * time.Microsecond
}

func (e rainbow) addUp(f, delta, max float64) float64 {
	f += delta
	for f >= max {
		f -= max
	}
	return f
}

func (e rainbow) Draw(canvas *ledclient.Canvas, p Parameters) {
	h, s, v := p.Color.Hsv()
	width, _ := canvas.Size()
	h += p.Angle
	hueStep := 140.0 / float64(width)

	FillFunc(canvas, func(x, y int, col colorful.Color) colorful.Color {
		hue := e.addUp(h, float64(x)*hueStep, 360.0)
		return colorful.Hsv(hue, s, v)
	})
}
