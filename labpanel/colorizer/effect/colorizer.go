package effect

import (
	"bufio"
	"github.com/dorkowscy/lyslix/lifx"
	"github.com/lucasb-eyer/go-colorful"
	"log"
	"net"
)

type Colorizer struct {
	Address uint64
	C       chan colorful.Color
}

const TEMPERATURE = 3200
const TRANSITION_MS = 90

// Given a float in the range 0..max, return a word in the range 0..65535.
func floattouint16(f float64, max float64) uint16 {
	return uint16(f / max * 65535)
}

func printHsl(c colorful.Color) {
	h, s, l := c.Clamped().Hsl()
	// log.Printf("H=%6.2f\tS=%6.2f\t\tL=%6.2f [clamped]", h, s, l)
	log.Printf("HSL\t= [%6.2f, %6.2f, %6.2f]", h, s, l)
}

func printLab(c colorful.Color) {
	l, a, b := c.Lab()
	log.Printf("L*a*b\t= [%6.2f, %6.2f, %6.2f]", l, a, b)
	// log.Printf("L=%6.2f\t*a=%6.2f\t*b=%6.2f", l, a, b)
}

// read colors from a channel and propagate on the lifx network
func (c *Colorizer) Run() {
	udp, err := net.DialUDP("udp4", nil, &net.UDPAddr{
		IP:   net.IPv4bcast,
		Port: 56700,
	})
	if err != nil {
		log.Fatalf("can't dial udp: %s", err)
	}
	sock := bufio.NewWriter(udp)

	for color := range c.C {
		//printLab(color)
		//printHsl(color)

		h, s, l := color.Clamped().Hsl()
		lc := []uint16{
			floattouint16(h, 360),
			floattouint16(s, 1),
			floattouint16(l, 1),
			TEMPERATURE,
		}

		packet := lifx.Packet{
			Header: lifx.Header{
				Frame: lifx.Frame{
					Addressable: true,
					Protocol:    1024,
				},
				FrameAddress: lifx.FrameAddress{
					Target: c.Address,
				},
				ProtocolHeader: lifx.ProtocolHeader{
					Type: lifx.MsgTypeSetColorMessage,
				},
			},
			Payload: &lifx.SetColorMessage{
				Duration: TRANSITION_MS,
				Color: lifx.HBSK{
					Hue:        lc[0],
					Saturation: lc[1],
					Brightness: lc[2],
					Kelvin:     lc[3],
				},
			},
		}

		packet.SetSize()

		err = packet.Write(sock)
		if err != nil {
			log.Println(err)
			sock.Reset(udp)
			continue
		}

		// Send data to the network
		err = sock.Flush()

		if err != nil {
			log.Println(err)
		}
	}
}
