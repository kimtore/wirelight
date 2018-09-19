package mqttcolor

import (
	"fmt"
	"math"
	"strconv"
	"strings"

	colorful "github.com/lucasb-eyer/go-colorful"
)

var BLACK colorful.Color

type State struct {
	On    bool
	Color colorful.Color
}

func (s State) Update(input string) (State, error) {
	if on, err := onoff(input); err == nil {
		s.On = on
		return s, nil
	}
	if c, err := hslstr2color(input); err == nil {
		s.On = true
		s.Color = c
		return s, nil
	}
	return s, fmt.Errorf("got '%s', expected ON, OFF, or R,G,B", input)
}

func (s *State) SwitchedColor() colorful.Color {
	if s.On {
		return s.Color
	}
	return BLACK
}

func onoff(s string) (bool, error) {
	if s == "ON" {
		return true, nil
	} else if s == "OFF" {
		return false, nil
	}
	return false, fmt.Errorf("not ON or OFF")
}

func hslstr2color(st string) (colorful.Color, error) {
	var h, s, l uint64
	var c colorful.Color
	var err error

	parts := strings.Split(st, ",")
	if len(parts) != 3 {
		return c, fmt.Errorf("expected: R,G,B")
	}

	if h, err = strconv.ParseUint(parts[0], 10, 64); err != nil {
		return c, err
	}
	if s, err = strconv.ParseUint(parts[1], 10, 64); err != nil {
		return c, err
	}
	if l, err = strconv.ParseUint(parts[2], 10, 64); err != nil {
		return c, err
	}

	c = colorful.Hsl(hsl2float(h, s, l))

	return c, nil
}

func squareFraction(number float64) float64 {
	return math.Ceil(number*number*0.5) / 10000.0
}

func hsl2float(h, s, l uint64) (float64, float64, float64) {
	return float64(h), float64(s) / 100.0, squareFraction(float64(l))
}
