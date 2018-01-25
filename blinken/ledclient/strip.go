package ledclient

import (
	"fmt"
	"time"

	"github.com/ambientsound/wirelight/blinken/lib"
	"github.com/ambientsound/wirelight/blinken/pb"
	"github.com/golang/protobuf/proto"
	"github.com/pebbe/zmq4"
)

// The serial is increased by one every time Blinken sends a LED update.
var serial uint64

// Strip represents a strip of LEDs.
type Strip struct {
	sock        *zmq4.Socket
	refreshRate uint64
	width       int
	height      int
	shutdown    chan int
}

// NewStrip returns Strip.
func NewStrip(sock *zmq4.Socket, width, height int, refreshRate uint64) *Strip {
	return &Strip{
		sock:        sock,
		refreshRate: refreshRate, // render all LEDs every 15th update
		width:       width,
		height:      height,
		shutdown:    make(chan int, 1),
	}
}

// rpcLED transfers one LED value to the remote server.
func (s *Strip) rpcLED(led *pb.LED) error {
	serial++
	led.Serial = serial
	led.Render = (serial%s.refreshRate == 0)

	payload, err := proto.Marshal(led)
	if err != nil {
		return fmt.Errorf("while generating protobuf payload: %s", err)
	}

	_, err = s.sock.SendBytes(payload, 0)
	if err != nil {
		return fmt.Errorf("while sending data using ZeroMQ: %s", err)
	}

	return nil
}

func cycleTime(freq int) time.Duration {
	return (1 * time.Second) / time.Duration(freq)
}

// Index returns the physical position of a single LED.
//
// The LEDs are configured in a zig-zag pattern, as drawn below. The LED server
// only knows a single strand, so we must perform the positional conversion here.
//
//     START.........................
//                                  |
//     ..............................
//     |
//     ...........................END
//
func (s *Strip) Index(x, y int) uint32 {
	if y%2 == 0 {
		return uint32(y*s.width + x)
	}
	return uint32(y*s.width + (s.width - x - 1))
}

// BitBlit transfers image data from an object implementing the Image interface
// to a remote LED server.
func (s *Strip) BitBlit(img *Canvas) error {
	led := &pb.LED{}
	for y := 0; y < s.height; y++ {
		for x := 0; x < s.width; x++ {
			led.Index = s.Index(x, y)
			c := img.At(x, y)
			led.Rgb = lib.RGBA(c.Clamped())
			err := s.rpcLED(led)
			if err != nil {
				return err
			}
		}
	}
	return nil
}

// Loop renders the LEDs periodically. This function never returns until
// Close() is called, so be sure to call it in a goroutine.
func (s *Strip) Loop(img *Canvas, freq int) {
	c := cycleTime(freq)
	for {
		select {
		case <-s.shutdown:
			return
		default:
			err := s.BitBlit(img)
			if err != nil {
				fmt.Printf("BitBlit: %s\n", err)
			}
			time.Sleep(c)
		}
	}
}

// Close turns all LEDs black and shuts down the rendering function.
func (s *Strip) Close() {
	s.shutdown <- 0
}
