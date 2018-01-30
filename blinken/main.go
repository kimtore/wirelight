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
	"github.com/spf13/viper"
)

type State struct {
	Color         colorful.Color
	MqttState     mqttlight.State
	FrontendState ws.State
}

func init() {
	flag.Parse()

	viper.SetConfigName("blinken")
	viper.SetConfigType("yaml")
	viper.AddConfigPath("/etc/")
	viper.AddConfigPath("$HOME/.blinken/")
	viper.AddConfigPath(".")
}

func mqttClient(address, username, password, topic, clientId string, messages chan []byte) (MQTT.Client, error) {
	flag.Parse()

	connOpts := MQTT.
		NewClientOptions().
		AddBroker(address).
		SetClientID(clientId).
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
	err := viper.ReadInConfig()
	if err != nil {
		fmt.Printf("%s\n", err)
		os.Exit(1)
	}

	// Set up ZeroMQ connection to LEDServer.
	ledserver := viper.GetString("ledserver.address")
	sock, err := ledclient.Socket(ledserver)
	if err != nil {
		fmt.Printf("Error: %s\n", err)
		os.Exit(1)
	}
	defer sock.Close()

	// Set up the LED strip writer.
	rows := viper.GetInt("height")
	cols := viper.GetInt("width")
	strip := ledclient.NewStrip(sock, rows, cols, uint64(rows*cols))
	canvas := ledclient.NewCanvas(rows, cols)
	defer strip.Close()

	// Send a continuous stream of LED updates through ZeroMQ.
	fps := viper.GetInt("ledserver.fps")
	fmt.Printf("Streaming LED updates to %s with %d FPS.\n", ledserver, fps)
	go strip.Loop(canvas, fps)

	// Set up MQTT client for MQTT JSON light support
	mqttMessages := make(chan []byte, 1024)
	_, err = mqttClient(
		viper.GetString("mqtt.address"),
		viper.GetString("mqtt.username"),
		viper.GetString("mqtt.password"),
		viper.GetString("mqtt.topic"),
		viper.GetString("mqtt.id"),
		mqttMessages,
	)
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

	// Set up program state
	state := State{}

	// termination
	terminate := make(chan int, 1)

	switchEffect := func(e effect.Effect) {
		terminate <- 1
		go effect.Run(e, terminate, canvas)
	}

	// Default effect is to switch off the lights.
	go effect.Run(effect.Effects["off"], terminate, canvas)

	// Loop through messages from the MQTT server and the frontend.
	for {
		select {
		case msg := <-mqttMessages:
			command, err := mqttlight.Unmarshal(msg)
			if err != nil {
				fmt.Printf("while decoding MQTT JSON message: %s\n", err)
				continue
			}
			state.MqttState = command
			ef := effect.Effects["solid"]
			if command.On() {
				state.Color = command.TransformColor(state.Color)
				ef.Palette["default"] = state.Color
			} else {
				ef.Palette["default"] = colorful.LinearRgb(0, 0, 0)
			}
			switchEffect(ef)

		case msg := <-wsMessages:
			state.FrontendState = msg
			ef := effect.Effects[msg.Effect]
			c := ws.MakeColor(msg)
			ef.Palette["default"] = c
			switchEffect(ef)

		case <-c:
			fmt.Printf("caught signal, exiting...\n")
			return
		}
	}
}
