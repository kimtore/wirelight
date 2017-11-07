package main

import (
	"bufio"
	"fmt"

	"github.com/ambientsound/wirelight/blinken/pb"
	"github.com/golang/protobuf/proto"
)

// The serial is increased by one every time Blinken sends a LED update.
var serial uint64

// Strip represents a strip of LEDs.
type Strip struct {
	writer *bufio.Writer
}

// NewStrip returns Strip.
func NewStrip(writer *bufio.Writer) *Strip {
	return &Strip{
		writer: writer,
	}
}

// rpcLED transfers one LED value to the remote server.
func (s *Strip) rpcLED(led *pb.LED) error {
	serial++
	led.Serial = serial

	payload, err := proto.Marshal(led)
	if err != nil {
		return fmt.Errorf("while generating protobuf payload: %s", err)
	}

	_, err = s.writer.Write(payload)
	if err != nil {
		return fmt.Errorf("while writing to buffered io: %s", err)
	}

	err = s.writer.Flush()
	if err != nil {
		return fmt.Errorf("while sending data on UDP socket: %s", err)
	}

	return nil
}

// Render instructs the server to render all the LEDs.
func (s *Strip) Render() error {
	led := &pb.LED{
		Render: true,
	}
	return s.rpcLED(led)
}

// Fill sets all the LEDs to one value.
func (s *Strip) Fill(color uint32) error {
	var i uint32

	led := &pb.LED{
		Rgb: color,
	}

	for i = 0; i < 60; i++ {
		led.Index = i
		err := s.rpcLED(led)
		if err != nil {
			return err
		}
	}

	return nil
}
