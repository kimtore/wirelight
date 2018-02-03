package effect

import (
	"time"

	"github.com/ambientsound/wirelight/blinken/ledclient"
)

type solid struct{}

func init() {
	Effects["solid"] = solid{}
}

func (e solid) Delay() time.Duration {
	return 1 * time.Second
}

func (e solid) Draw(canvas *ledclient.Canvas, p Parameters) {
	Fill(canvas, p.Color)
}
