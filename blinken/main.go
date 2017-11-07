// Blinken by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
//
// This program sends LED updates to a LEDServer using Google Protobuf messages.

package main

import (
	"bufio"
	"fmt"
	"net"
	"os"
	"time"

	"github.com/ambientsound/wirelight/blinken/pb"
	"github.com/golang/protobuf/proto"
	flag "github.com/ogier/pflag"
)

var (
	addr = flag.String("address", "blinkt:1230", "LEDServer address")
)

func WriteLED(writer *bufio.Writer, led *pb.LED) error {
	payload, err := proto.Marshal(led)
	if err != nil {
		return fmt.Errorf("while generating protobuf payload: %s", err)
	}

	_, err = writer.Write(payload)
	if err != nil {
		return fmt.Errorf("while writing to buffered io: %s", err)
	}

	err = writer.Flush()
	if err != nil {
		return fmt.Errorf("while sending data on UDP socket: %s", err)
	}

	return nil
}

func Fill(writer *bufio.Writer, color uint32) error {
	var i uint32

	led := &pb.LED{
		Rgb: color,
	}

	for i = 0; i < 60; i++ {
		led.Index = i
		err := WriteLED(writer, led)
		if err != nil {
			return err
		}
	}

	return nil
}

func Render(writer *bufio.Writer) {
	writer.Write([]byte{'F'})
	writer.Flush()
}

func main() {
	sock, err := net.Dial("udp", *addr)
	if err != nil {
		fmt.Printf("while dialing LEDServer at %s: %s\n", *addr, err)
		os.Exit(1)
	}

	fmt.Printf("Sending datagrams to %s\n", *addr)

	writer := bufio.NewWriter(sock)

	var color uint32
	for {
		if color == 0 {
			color = 255
		} else {
			color = 0
		}
		//color = rand.Uint32() >> 16
		err = Fill(writer, color)
		if err != nil {
			fmt.Printf("%s\n", err)
			os.Exit(1)
		}
		Render(writer)

		time.Sleep((1 * time.Second) / 24) // 24 fps
	}
}

func init() {
	flag.Parse()
}
