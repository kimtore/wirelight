// Blinken by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
//
// This program sends LED updates to a LEDServer using Google Protobuf messages.

package main

import (
	"fmt"
	"image"
	"os"
	"os/signal"
	"time"

	flag "github.com/ogier/pflag"
	"github.com/pebbe/zmq4"
)

var (
	addr = flag.String("ledserver", "tcp://blinkt:1230", "LEDServer address")
	freq = flag.Int("freq", 24, "Update frequency")
	cols = flag.Int("cols", 4, "Number of LED strips")
	rows = flag.Int("rows", 60, "Number of LEDs in one strip")
)

func init() {
	flag.Parse()
}

func cycleTime(freq int) time.Duration {
	return (1 * time.Second) / time.Duration(freq)
}

func zmqSocket(address string) (*zmq4.Socket, error) {
	ctx, err := zmq4.NewContext()
	if err != nil {
		return nil, fmt.Errorf("while creating ZeroMQ context: %s\n", err)
	}

	sock, err := ctx.NewSocket(zmq4.PUB)
	if err != nil {
		return nil, fmt.Errorf("while creating ZeroMQ socket: %s\n", err)
	}

	err = sock.Connect(*addr)
	if err != nil {
		return nil, fmt.Errorf("while connecting to %s: %s\n", *addr, err)
	}

	return sock, nil
}

func main() {
	fmt.Printf("Sending LED updates to %s.\n", *addr)

	sock, err := zmqSocket(*addr)
	if err != nil {
		fmt.Printf("Error: %s\n", err)
		os.Exit(1)
	}
	defer sock.Close()

	strip := NewStrip(sock, *rows, *cols, uint64((*rows)*(*cols)))
	rect := image.Rectangle{
		Min: image.Point{0, 0},
		Max: image.Point{*rows, *cols},
	}
	canvas := image.NewRGBA(rect)

	go strip.Loop(canvas, *freq)
	go split(canvas)

	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt)
	<-c
	fmt.Printf("caught signal, exiting...\n")
	black(canvas)
	time.Sleep(time.Millisecond * 10)
	os.Exit(0)
}
