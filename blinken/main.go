// Blinken by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
//
// This program sends LED updates to a LEDServer using Google Protobuf messages.

package main

import (
	"crypto/tls"
	"fmt"
	"os"
	"os/signal"

	"github.com/ambientsound/wirelight/blinken/effect"
	"github.com/ambientsound/wirelight/blinken/ledclient"
	"github.com/ambientsound/wirelight/blinken/mqttlight"
	"github.com/ambientsound/wirelight/blinken/ws"
	MQTT "github.com/eclipse/paho.mqtt.golang"
	colorful "github.com/lucasb-eyer/go-colorful"
	flag "github.com/ogier/pflag"
)

const CLIENT_ID string = "blinken"

var (
	ledServerAddress  = flag.String("ledserver", "tcp://blinkt:1230", "LEDServer address")
	freq              = flag.Int("freq", 24, "Update frequency")
	cols              = flag.Int("cols", 4, "Number of LED strips")
	rows              = flag.Int("rows", 60, "Number of LEDs in one strip")
	mqttServerAddress = flag.String("mqtt", "tcp://127.0.0.1:1883", "The full url of the MQTT server to connect to")
	mqttTopic         = flag.String("topic", "powerlamp/set", "Topic to subscribe to")
	mqttUsername      = flag.String("username", "", "A username to authenticate to the MQTT server")
	mqttPassword      = flag.String("password", "", "Password to match username")
)

func init() {
	flag.Parse()
}

func mqttClient(address, username, password, topic string, messages chan []byte) (MQTT.Client, error) {
	flag.Parse()

	connOpts := MQTT.
		NewClientOptions().
		AddBroker(address).
		SetClientID(CLIENT_ID).
		SetCleanSession(true).
		SetAutoReconnect(true)

	if username != "" {
		connOpts.SetUsername(username)
		if password != "" {
			connOpts.SetPassword(password)
		}
	}
	//tlsConfig := &tls.Config{InsecureSkipVerify: true, ClientAuth: tls.NoClientCert}
	tlsConfig := &tls.Config{}
	connOpts.SetTLSConfig(tlsConfig)

	connOpts.OnConnect = func(c MQTT.Client) {
		token := c.Subscribe(topic, byte(0), func(client MQTT.Client, message MQTT.Message) {
			messages <- message.Payload()
		})
		token.Wait()
		if token.Error() != nil {
			panic(token.Error())
		}
	}

	client := MQTT.NewClient(connOpts)
	token := client.Connect()

	if token.Wait() && token.Error() != nil {
		return nil, token.Error()
	}

	return client, nil
}

func main() {
	// Set up ZeroMQ connection to LEDServer.
	sock, err := ledclient.Socket(*ledServerAddress)
	if err != nil {
		fmt.Printf("Error: %s\n", err)
		os.Exit(1)
	}
	defer sock.Close()

	// Set up the LED strip writer.
	strip := ledclient.NewStrip(sock, *rows, *cols, uint64((*rows)*(*cols)))
	/*
		rect := image.Rectangle{
			Min: image.Point{0, 0},
			Max: image.Point{*rows, *cols},
		}
	*/
	canvas := ledclient.NewCanvas(*rows, *cols)
	defer strip.Close()

	// Send a continuous stream of LED updates through ZeroMQ.
	fmt.Printf("Sending LED updates to %s.\n", *ledServerAddress)
	go strip.Loop(canvas, *freq)

	// Set up MQTT client for MQTT JSON light support
	mqttMessages := make(chan []byte, 1024)
	_, err = mqttClient(*mqttServerAddress, *mqttUsername, *mqttPassword, *mqttTopic, mqttMessages)
	if err != nil {
		fmt.Printf("Error: %s\n", err)
		os.Exit(1)
	}

	// Set up Websockets server
	wsMessages := make(chan ws.State, 1024)
	go ws.Serve("0.0.0.0:8011", "/", wsMessages)

	// Set up signal handler
	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt)

	// Cache the last used color.
	oldColor := colorful.Color{}

	// termination
	var ef effect.Effect
	terminate := make(chan int, 1)

	// Default effect is to switch off the lights.
	ef = effect.Effects["off"]
	go effect.Run(ef, terminate, canvas)

	// Loop through MQTT messages.
	for {
		select {
		case msg := <-mqttMessages:
			command, err := mqttlight.Unmarshal(msg)
			//fmt.Printf("%+v\n", command)
			if err != nil {
				fmt.Printf("while decoding JSON message: %s\n", err)
				continue
			}
			if command.On() {
				newColor := command.TransformColor(oldColor).Clamped()
				h, c, l := newColor.Hcl()
				fmt.Printf("%.2f %.2f %.2f\n", h, c, l)
				effect.Fill(canvas, newColor)
				oldColor = newColor
			} else {
				effect.Fill(canvas, colorful.Color{})
			}
			//fmt.Printf("This is a message of type %+v.\n", command.Type())

		case msg := <-wsMessages:
			terminate <- 1
			ef = effect.Effects[msg.Effect]
			c := ws.MakeColor(msg)
			ef.Palette["default"] = c
			go effect.Run(ef, terminate, canvas)

		case <-c:
			fmt.Printf("caught signal, exiting...\n")
			return
		}
	}
}
