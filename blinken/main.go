// Blinken by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
//
// This program sends LED updates to a LEDServer using Google Protobuf messages.

package main

import (
	"bufio"
	"fmt"
	"image"
	"image/color"
	"net"
	"os"
	"time"

	flag "github.com/ogier/pflag"
)

var (
	addr   = flag.String("address", "blinkt:1230", "LEDServer address")
	freq   = flag.Int("freq", 8, "Updates per second")
	render = flag.Int64("render", 240, "Render strip every N led update")
)

func init() {
	flag.Parse()
}

func cycleTime(freq int) time.Duration {
	return (1 * time.Second) / time.Duration(freq)
}

func main() {
	fmt.Printf("Sending UDP datagrams to %s.\n", *addr)

	sock, err := net.Dial("udp", *addr)
	if err != nil {
		fmt.Printf("while dialing LEDServer at %s: %s\n", *addr, err)
		os.Exit(1)
	}

	writer := bufio.NewWriter(sock)
	strip := NewStrip(writer, 240, 1, uint64(*render))
	rect := image.Rectangle{
		Min: image.Point{0, 0},
		Max: image.Point{240, 1},
	}
	canvas := image.NewRGBA(rect)

	go strip.Loop(canvas, *freq)
	northernLights(canvas)
}

func fill(canvas *image.RGBA, col color.Color) {
	b := canvas.Bounds()
	for x := b.Min.X; x < b.Max.X; x++ {
		for y := b.Min.Y; y < b.Max.Y; y++ {
			canvas.Set(x, y, col)
		}
	}
}
