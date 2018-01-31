package effect

import (
	"time"
)

type solid struct{}

func init() {
	Effects["solid"] = solid{}
}

func (e solid) Delay() time.Duration {
	return 1 * time.Second
}

func (e solid) Draw(p Parameters) {
	Fill(p.Canvas, p.Color)
}
