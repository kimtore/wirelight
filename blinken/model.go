package main

import (
	"image"
	"image/color"

	colorful "github.com/lucasb-eyer/go-colorful"
)

// Colorspace as a separate module?
type Colorspace int

const (
	HCL Colorspace = iota
	HSL
	HSV
	Lab
	Luv
	RGB
)

func ColorspaceGradient(s Colorspace) func(a, b, c float64) colorful.Color {
	switch s {
	default:
		return colorful.Hcl
	}
}

type Canvas struct {
	Rect    image.Rectangle
	Effects []Effect
}

type Effect struct {
	Canvas Canvas
}

type Gradient struct {
	Colors []color.Color
	Space  Colorspace
}
