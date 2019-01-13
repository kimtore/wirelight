package effect

import (
	"github.com/ambientsound/wirelight/labpanel/colorizer/panel"
	"github.com/lucasb-eyer/go-colorful"
)

// The distributor's job is to translate Panel messages to individual bulb colors.
type Distributor struct {
	Colorizers []Colorizer
	C          chan panel.Panel
}

const MAX_ADJUST_ANGLE = 360.0

func floattoangle(f float64) float64 {
	return f * MAX_ADJUST_ANGLE
}

func (d *Distributor) Run() {
	numBulbs := float64(len(d.Colorizers))

	for p := range d.C {
		// starting color
		c := p.Color()
		h, s, l := c.Clamped().Hsl()

		// we will draw colors within a subset of the color circle.
		// angle represents the hue range.
		angle := floattoangle(p.Adjust)

		// color distance from one bulb to the next
		distance := angle / numBulbs

		// adjust start hue so that target hue is in the middle
		h -= distance * 2

		for _, colorizer := range d.Colorizers {
			c = colorful.Hsl(h, s, l)
			colorizer.C <- c
			h += distance
			if h > 360.0 {
				h -= 360.0
			}
		}
	}
}
