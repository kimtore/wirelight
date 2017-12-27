package effect

import (
	"time"

	colorful "github.com/lucasb-eyer/go-colorful"
)

func init() {
	Effects["solid"] = Effect{
		Name:     "Solid color",
		Function: solid,
		Delay:    1 * time.Second,
		Palette: Palette{
			"default": colorful.Hcl(0, 0, 0),
		},
	}
}

func solid(e Effect) Effect {
	Fill(e.Canvas, e.Palette["default"])
	return e
}
