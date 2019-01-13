package main

import (
	"encoding/hex"
	"github.com/ambientsound/wirelight/labpanel/colorizer/effect"
	"github.com/ambientsound/wirelight/labpanel/colorizer/panel"
	"github.com/dorkowscy/lyslix/lifx"
	"github.com/eclipse/paho.mqtt.golang"
	"github.com/lucasb-eyer/go-colorful"
	flag "github.com/spf13/pflag"
	"log"
	"os"
	"os/signal"
	"strings"
)

const PROGNAME = "colorizer"

func main() {
	address := flag.String("address", "tcps://localhost:1883", "host")
	username := flag.String("username", "", "username")
	password := flag.String("password", "", "password")
	clientId := flag.String("clientId", PROGNAME, "clientId")
	topic := flag.String("topic", "", "topic")
	bulbs := flag.StringSlice("bulbs", []string{}, "lifx bulb mac addresses")

	flag.Parse()

	messages := make(chan panel.Panel, 64)
	signals := make(chan os.Signal, 1)
	colorizers := make([]effect.Colorizer, len(*bulbs))
	signal.Notify(signals, os.Interrupt)

	// Set up an effect processor for each of the bulbs
	for i, mac := range *bulbs {
		mac = strings.Replace(mac, ":", "", 5)
		byteAddress, err := hex.DecodeString(mac)
		if err != nil {
			log.Fatalf("unable to decode mac address: %s", err)
		}
		if len(byteAddress) != 6 {
			log.Fatalf("mac address does not have six bytes")
		}
		colorizers[i] = effect.Colorizer{
			Address: lifx.MACAdressToFrameAddress(byteAddress),
			C:       make(chan colorful.Color, 64),
		}
		go colorizers[i].Run()
	}

	// Instantiate a distribution function that will generate
	// an unique color for each of the lights.
	distributor := effect.Distributor{
		Colorizers: colorizers,
		C:          make(chan panel.Panel, 64),
	}
	go distributor.Run()

	// Instantiate a MQTT client. This is where all the color information arrives.
	opts := mqtt.
		NewClientOptions().
		AddBroker(*address).
		SetUsername(*username).
		SetPassword(*password).
		SetClientID(*clientId).
		SetCleanSession(true).
		SetAutoReconnect(true)

	opts.OnConnect = func(c mqtt.Client) {
		token := c.Subscribe(*topic, byte(0), func(client mqtt.Client, message mqtt.Message) {
			payload := string(message.Payload())
			messages <- panel.Parse(payload)
		})
		token.Wait()
		if token.Error() != nil {
			log.Fatal(token.Error())
		}
	}

	client := mqtt.NewClient(opts)
	token := client.Connect()

	if token.Wait() && token.Error() != nil {
		log.Fatal(token.Error())
	}

	log.Printf("Connected to MQTT server %s on topic %s.\n", *address, *topic)

	for {
		select {
		case msg := <-messages:
			distributor.C <- msg
		case <-signals:
			log.Printf("caught signal, exiting...\n")
			return
		}
	}
}
