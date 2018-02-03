package effect

import (
	"math"
	"time"

	"github.com/ambientsound/wirelight/blinken/ledclient"
	"github.com/ambientsound/wirelight/blinken/lib"
	colorful "github.com/lucasb-eyer/go-colorful"
)

type wave struct{}

func init() {
	Effects["wave"] = wave{}
}

func (e wave) Delay() time.Duration {
	return 400 * time.Microsecond
}

func (e wave) Draw(canvas *ledclient.Canvas, p Parameters) {
	h, s, v := p.Color.Hsv()
	width, _ := canvas.Size()
	xstep := 180.0 / float64(width) // wave length equals one strip length
	waveSine := p.Angle - 180.0

	FillFunc(canvas, func(x, y int, col colorful.Color) colorful.Color {
		lumAngle := waveSine + (float64(x) * xstep)
		sin := (1 + math.Sin(lib.Rad(lumAngle))) / 4
		val := v + sin
		return colorful.Hsv(h, s, val)
	})
}
