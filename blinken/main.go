// Blinken by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
//
// This program sends LED updates to a LEDServer using Google Protobuf messages.

package main

import (
	"bufio"
	"fmt"
	"image"
	"image/color"
	"math"
	"math/rand"
	"net"
	"os"
	"time"

	"github.com/ambientsound/wirelight/blinken/pb"
	colorful "github.com/lucasb-eyer/go-colorful"
	flag "github.com/ogier/pflag"
)

var (
	addr   = flag.String("address", "blinkt:1230", "LEDServer address")
	freq   = flag.Int("freq", 24, "Updates per second")
	render = flag.Int64("render", 30, "Render strip every N led update")
)

func init() {
	flag.Parse()
}

func cycleTime(freq int) time.Duration {
	return (1 * time.Second) / time.Duration(freq)
}

// Fill sets all the LEDs to one value.
func (s *Strip) Fill(color uint32) error {
	led := &pb.LED{
		Rgb: color,
	}

	for i := 0; i < s.width; i++ {
		led.Index = uint32(i)
		err := s.rpcLED(led)
		if err != nil {
			return err
		}
	}

	return nil
}

func main() {
	fmt.Printf("Sending UDP datagrams to %s.\n", *addr)

	sock, err := net.Dial("udp", *addr)
	if err != nil {
		fmt.Printf("while dialing LEDServer at %s: %s\n", *addr, err)
		os.Exit(1)
	}

	writer := bufio.NewWriter(sock)
	strip := NewStrip(writer, 60, 1, uint64(*render))
	rect := image.Rectangle{
		Min: image.Point{0, 0},
		Max: image.Point{60, 1},
	}
	canvas := image.NewRGBA(rect)

	go strip.Loop(canvas, *freq)
	northernLights(canvas)
}

func Rad(d float64) float64 {
	const x = math.Pi / 180
	return d * x
}

func fill(canvas *image.RGBA, col color.Color) {
	b := canvas.Bounds()
	for x := b.Min.X; x < b.Max.X; x++ {
		for y := b.Min.Y; y < b.Max.Y; y++ {
			canvas.Set(x, y, col)
		}
	}
}

func northernLights(canvas *image.RGBA) {
	b := canvas.Bounds()
	old := make([]colorful.Color, b.Max.X*b.Max.Y)
	for {
		for angle := 0.0; angle < 360.0; angle++ {
			for x := b.Min.X; x < b.Max.X; x++ {
				for y := b.Min.Y; y < b.Max.Y; y++ {
					i := (y * b.Max.X) + x
					col := colorful.Hsl(angle+rand.Float64()*50.0, 1, rand.Float64()*0.6)
					step := col.BlendHcl(old[i], 0.92).Clamped()
					canvas.Set(x, y, step)
					old[i] = step
				}
			}
			time.Sleep(time.Millisecond * 100)
		}
	}
}

func white(canvas *image.RGBA) {
	for {
		hue := rand.Float64() * 360.0
		for deg := 0.0; deg <= 180.0; deg += 1 {
			l := math.Sin(Rad(deg))
			col := colorful.Hsv(hue, 1.0, l*0.5).Clamped()
			fill(canvas, col)
			time.Sleep(time.Microsecond * 1500)
		}
		time.Sleep(time.Millisecond * 185)
	}
}

func gradients(canvas *image.RGBA) {
	var h, c, l float64
	h = 0.0
	c = 0.8
	l = 0.5
	_, _ = c, l
	src := colorful.Hsv(h, 1, 1)
	dst := colorful.Hsv(h, 1, 1)

	for {
		src = dst
		h += 30
		if h >= 360 {
			h = 0
		}
		dst = colorful.Hsv(h, 1, 1)
		fmt.Printf("hue=%.2f, blend %#v %#v\n", h, src, dst)

		// interpolate between the two colors.
		for n := 0.0; n < 1.0; n += 0.01 {
			col := src.BlendHcl(dst, n).Clamped()
			fill(canvas, col)
			time.Sleep(time.Millisecond * 20)
		}
	}
}

func wheelHCL(canvas *image.RGBA) {
	var h float64
	for {
		h += 1
		if h > 360 {
			h = 0
		}
		col := colorful.Hcl(h, 0.2, 0).Clamped()
		fill(canvas, col)
		time.Sleep(time.Millisecond * 10)
	}
}

func wheelHSV(canvas *image.RGBA) {
	var h float64
	for {
		h += 1
		if h > 360 {
			h = 0
		}
		col := colorful.Hsv(h, 1, 1)
		fill(canvas, col)
		time.Sleep(time.Millisecond * 10)
	}
}
