package effect

import (
	"time"

	colorful "github.com/lucasb-eyer/go-colorful"
)

var rainbowSine float64 = 0.0

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

func (e rainbow) Draw(p Parameters) {
	h, s, v := p.Color.Hsv()
	width, _ := p.Canvas.Size()
	h += waveSine
	hueStep := 140.0 / float64(width)

	FillFunc(p.Canvas, func(x, y int, col colorful.Color) colorful.Color {
		hue := e.addUp(h, float64(x)*hueStep, 360.0)
		return colorful.Hsv(hue, s, v)
	})

	waveSine += 0.1
	if waveSine >= 360.0 {
		waveSine = 0
	}
}
