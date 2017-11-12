// Blinken by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
//
// This program sends LED updates to a LEDServer using Google Protobuf messages.

package main

import (
	"bufio"
	"fmt"
	"image"
	"image/color"
	"math/rand"
	"net"
	"os"
	"time"

	"github.com/ambientsound/wirelight/blinken/pb"
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
	moo(canvas)
}

func fill(canvas *image.RGBA, col color.Color) {
	b := canvas.Bounds()
	for x := b.Min.X; x < b.Max.X; x++ {
		for y := b.Min.Y; y < b.Max.Y; y++ {
			canvas.Set(x, y, col)
		}
	}
}

func moo(canvas *image.RGBA) {
	for {
		col := color.RGBA{
			R: uint8(rand.Int()),
			G: uint8(rand.Int()),
			B: uint8(rand.Int()),
			A: 0,
		}
		fill(canvas, col)
		fmt.Printf("Filled canvas with %#v\n", col)
		time.Sleep(time.Second * 1)
	}
}
