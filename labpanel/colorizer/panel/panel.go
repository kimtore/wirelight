package panel

import (
	"github.com/lucasb-eyer/go-colorful"
	"math"
	"strconv"
	"strings"
)

// Panel represents the state of the Labpanel hardware board.
// The values of L, A and B correspond to L*a*b colors.
type Panel struct {
	L      float64
	A      float64
	B      float64
	Adjust float64
}

func (p Panel) Color() colorful.Color {
	return colorful.Lab(p.L, p.A, p.B)
}

// Convert a 10-bit integer to a floating point number between 0.0-1.0.
func ToFloat(i int) float64 {
	return float64(i) / 1024
}

// Return the average between two float values.
func Avg(min, max float64) float64 {
	return max/2 - min/2 + min
}

// The scale is not linear. The closest estimate so far
// is the average between the square and cubic roots of x.
func Scale(x float64) float64 {
	if x == 0 {
		return x
	}
	return Avg(x/math.Sqrt(x), x/math.Cbrt(x))
}

// Convert from ranges 0.0-1.0 into native L*a*b ranges.
// https://stackoverflow.com/questions/19099063/what-are-the-ranges-of-coordinates-in-the-cielab-color-space
func New(L, a, b, adjust float64) Panel {
	return Panel{
		L,
		((2*a - 1) + 0.86185) / 1.84439,
		((2*b - 1) + 1.07863) / 2.02345,
		adjust,
	}
}

// Parse a string containing four 10-bit numbers into a Panel structure.
func Parse(str string) Panel {
	state := strings.SplitN(str, " ", 5)

	L, _ := strconv.Atoi(state[0])
	a, _ := strconv.Atoi(state[1])
	b, _ := strconv.Atoi(state[2])
	adjust, _ := strconv.Atoi(state[3])

	return New(
		Scale(ToFloat(L)),
		Scale(ToFloat(a)),
		Scale(ToFloat(b)),
		Scale(ToFloat(adjust)),
	)
}
