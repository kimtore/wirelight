package mqttlight

import (
	"crypto/tls"
	"flag"

	MQTT "github.com/eclipse/paho.mqtt.golang"
)

func New(address, username, password, topic, clientId string, messages chan []byte) (MQTT.Client, error) {
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
