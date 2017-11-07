// Blinken by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
//
// This program sends LED updates to a LEDServer using Google Protobuf messages.

package main

import (
	"bufio"
	"fmt"
	"math/rand"
	"net"
	"os"
	"time"

	flag "github.com/ogier/pflag"
)

var (
	addr = flag.String("address", "blinkt:1230", "LEDServer address")
	freq = flag.Int("freq", 4, "Frequency of updates")
)

func init() {
	flag.Parse()
}

func main() {
	fmt.Printf("Sending UDP datagrams to %s.\n", *addr)

	sock, err := net.Dial("udp", *addr)
	if err != nil {
		fmt.Printf("while dialing LEDServer at %s: %s\n", *addr, err)
		os.Exit(1)
	}

	writer := bufio.NewWriter(sock)
	strip := NewStrip(writer)

	err = blink(strip, *freq)
	fmt.Printf("%s\n", err)
}

func blink(s *Strip, freq int) error {
	var color uint32
	duration := (1 * time.Second) / time.Duration(freq)

	for {
		if color == 0 {
			color = rand.Uint32() >> 8
		} else {
			color = 0
		}
		err := s.Fill(color)
		if err != nil {
			return err
		}
		s.Render()

		time.Sleep(duration)
	}

	return nil
}
