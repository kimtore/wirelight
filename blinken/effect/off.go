package effect

import (
	"time"

	colorful "github.com/lucasb-eyer/go-colorful"
)

type off struct{}

func init() {
	Effects["off"] = off{}
}

func (e off) Delay() time.Duration {
	return 1 * time.Hour
}

func (e off) Draw(p Parameters) {
	Fill(p.Canvas, colorful.LinearRgb(0, 0, 0))
}
