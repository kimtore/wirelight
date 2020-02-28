package main

import (
	"encoding/json"
	"os"
	"os/signal"
	"strings"
	"syscall"

	"github.com/ambientsound/wirelight/servo/light"
	mqtt "github.com/eclipse/paho.mqtt.golang"
	log "github.com/sirupsen/logrus"
	flag "github.com/spf13/pflag"
	"github.com/spf13/viper"
)

type Config struct {
	Lights   []string
	Topic    string
	Server   string
	Username string
	Password string
	LogLevel string
}

var cfg Config

var handlers = map[string]mqtt.MessageHandler{
	servoTopic: servoHandler,
}

const servoTopic = "servo/+/light/neopixels/command"

func fatal(err error) {
	log.Printf("fatal: %s", err)
	os.Exit(1)
}

func init() {
	viper.SetEnvPrefix("SERVO")
	viper.SetEnvKeyReplacer(strings.NewReplacer("-", "_", ".", "_"))
	viper.AutomaticEnv()

	flag.StringVar(&cfg.Topic, "topic", "+/light/neopixels/state", "Topics to subscribe to")
	flag.StringVar(&cfg.Server, "server", "tcp://127.0.0.1:1883", "The full URL of the MQTT server to connect to")
	flag.StringVar(&cfg.Username, "username", "", "A username to authenticate to the MQTT server")
	flag.StringVar(&cfg.Password, "password", "", "Password to match username")
	flag.StringVar(&cfg.LogLevel, "log-level", "INFO", "Log level, from TRACE to ERROR")
	flag.Parse()

	err := viper.BindPFlags(flag.CommandLine)
	if err != nil {
		fatal(err)
	}

	err = viper.Unmarshal(&cfg)
	if err != nil {
		fatal(err)
	}

	lv, err := log.ParseLevel(cfg.LogLevel)
	if err != nil {
		fatal(err)
	}

	log.SetLevel(lv)
}

func printMessage(message mqtt.Message) {
	log.Tracef("< %s: %s", message.Topic(), string(message.Payload()))
}

func espHandler(client mqtt.Client, message mqtt.Message) {
	log.Tracef("espHandler")
	printMessage(message)
	lt := &light.Esp{}
	err := json.Unmarshal(message.Payload(), lt)
	if err != nil {
		log.Error(err)
		return
	}
	rgb, err := lt.RGB()
	if err != nil {
		log.Error(err)
		return
	}
	destination := "servo/" + message.Topic()
	if token := client.Publish(destination, 0, false, *rgb); token.Wait() && token.Error() != nil {
		fatal(token.Error())
	}
	log.Tracef("> %s: %s", destination, *rgb)
}

func servoHandler(client mqtt.Client, message mqtt.Message) {
	log.Tracef("servoHandler")
	printMessage(message)
	lt, err := light.Parse(string(message.Payload()))
	if err != nil {
		log.Error(err)
		return
	}
	relay, err := lt.Serialize()
	if err != nil {
		fatal(err)
	}
	destination := strings.Replace(message.Topic(), "servo/", "", 1)
	if token := client.Publish(destination, 0, false, relay); token.Wait() && token.Error() != nil {
		fatal(token.Error())
	}
	log.Tracef("> %s: %s", destination, string(relay))
}

func subscribeTo(client mqtt.Client, topics ...string) error {
	for _, topic := range topics {
		if token := client.Subscribe(topic, 0, handlers[topic]); token.Wait() && token.Error() != nil {
			return token.Error()
		}
		log.Infof("subscribed to %s", topic)
	}
	return nil
}

func main() {
	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)

	handlers[cfg.Topic] = espHandler

	opts := mqtt.NewClientOptions().
		AddBroker(cfg.Server).
		SetUsername(cfg.Username).
		SetPassword(cfg.Password).
		SetAutoReconnect(true).
		SetOnConnectHandler(func(client mqtt.Client) {
			err := subscribeTo(client, cfg.Topic, servoTopic)
			if err != nil {
				fatal(err)
			}
		})

	client := mqtt.NewClient(opts)
	if token := client.Connect(); token.Wait() && token.Error() != nil {
		fatal(token.Error())
	}

	log.Printf("connected to %s", cfg.Server)

	<-c
}
