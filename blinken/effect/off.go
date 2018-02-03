package effect

import (
	"time"

	"github.com/ambientsound/wirelight/blinken/ledclient"
	colorful "github.com/lucasb-eyer/go-colorful"
)

type off struct{}

func init() {
	Effects["off"] = off{}
}

func (e off) Delay() time.Duration {
	return 1 * time.Hour
}

func (e off) Draw(canvas *ledclient.Canvas, p Parameters) {
	Fill(canvas, colorful.LinearRgb(0, 0, 0))
}
