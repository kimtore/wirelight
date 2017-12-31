package effect

import (
	"time"

	colorful "github.com/lucasb-eyer/go-colorful"
)

func init() {
	Effects["off"] = Effect{
		Name:     "Darkness",
		Function: off,
		Delay:    10000 * time.Hour,
		Palette:  Palette{},
	}
}

func off(e Effect) Effect {
	Fill(e.Canvas, colorful.LinearRgb(0, 0, 0))
	return e
}
