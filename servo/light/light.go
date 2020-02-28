package light

import (
	"encoding/json"
	"fmt"
	"strconv"
	"strings"
)

type RGB struct {
	R uint8 `json:"r"`
	G uint8 `json:"g"`
	B uint8 `json:"b"`
}

func (rgb RGB) Brightness() uint8 {
	switch {
	case rgb.R > rgb.G && rgb.R > rgb.B:
		return rgb.R
	case rgb.G > rgb.R || rgb.G > rgb.B:
		return rgb.G
	default:
		return rgb.B
	}
}

type Esp struct {
	Brightness *uint8  `json:"brightness,omitempty"`
	ColorTemp  *uint16 `json:"color_temp,omitempty"`
	Color      *RGB    `json:"color,omitempty"`
	Effect     *string `json:"effect,omitempty"`
	State      *string `json:"state,omitempty"`
	Transition *int    `json:"transition,omitempty"`
	WhiteValue *int    `json:"white_value,omitempty"`
}

func Parse(rgb string) (*Esp, error) {
	c := strings.Split(rgb, ",")
	if len(c) != 3 {
		return nil, fmt.Errorf("wrong format, expecting three comma-separated values")
	}
	p := make([]uint8, len(c))
	for i := range c {
		j, err := strconv.ParseUint(c[i], 10, 8)
		if err != nil {
			return nil, err
		}
		p[i] = uint8(j)
	}
	return &Esp{
		Color: &RGB{
			R: p[0],
			G: p[1],
			B: p[2],
		},
	}, nil
}

func uint8ptr(i uint8) *uint8 {
	return &i
}

func stringptr(s string) *string {
	return &s
}

func (e *Esp) Serialize() ([]byte, error) {
	e.Brightness = uint8ptr(e.Color.Brightness())
	e.State = stringptr("ON")
	return json.Marshal(e)
}

func (e *Esp) RGB() (*string, error) {
	if e.Color == nil {
		return nil, fmt.Errorf("no color information")
	}
	rgb := fmt.Sprintf("%d,%d,%d", e.Color.R, e.Color.G, e.Color.B)
	return &rgb, nil
}
