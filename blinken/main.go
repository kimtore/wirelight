// Blinken by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
//
// This program sends LED updates to a LEDServer using Google Protobuf messages.

package main

import (
	"fmt"
	"os"
	"os/signal"

	"github.com/ambientsound/wirelight/blinken/effect"
	"github.com/ambientsound/wirelight/blinken/ledclient"
	"github.com/ambientsound/wirelight/blinken/mqttlight"
	"github.com/ambientsound/wirelight/blinken/ws"
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
	width := viper.GetInt("width")
	height := viper.GetInt("height")
	strip := ledclient.NewStrip(sock, width, height, uint64(width*height))
	canvas := ledclient.NewCanvas(width, height)
	defer strip.Close()

	// Send a continuous stream of LED updates through ZeroMQ.
	fps := viper.GetInt("ledserver.fps")
	fmt.Printf("Streaming LED updates to %s with %d FPS.\n", ledserver, fps)
	go strip.Loop(canvas, fps)

	// Set up MQTT client for MQTT JSON light support
	mqttMessages := make(chan []byte, 1024)
	_, err = mqttlight.New(
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

	// Effect communication
	effectPipeline := make(chan effect.Parameters, 32)
	terminate := make(chan int, 1)
	params := effect.Parameters{
		Name: "solid",
	}

	// Default effect is to switch off the lights.
	go effect.Run(canvas, effectPipeline, terminate)
	effectPipeline <- params

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
			if command.On() {
				state.Color = command.TransformColor(state.Color)
				params.Color = state.Color
			} else {
				params.Color = colorful.LinearRgb(0, 0, 0)
			}
			params.Name = "solid"
			effectPipeline <- params

		case msg := <-wsMessages:
			//fmt.Printf("%+v\n", msg)
			state.FrontendState = msg
			c := ws.MakeColor(msg)
			params.Name = msg.Effect
			params.Color = c
			effectPipeline <- params

		case <-c:
			fmt.Printf("caught signal, exiting...\n")
			return
		}
	}
}
