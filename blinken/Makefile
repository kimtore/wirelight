# Blinken by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
#
# This program sends Google Protobuf messages to LEDServer.

.PHONY: all proto

all:
	go build

dep:
	go get -u github.com/eclipse/paho.mqtt.golang
	go get -u github.com/golang/protobuf/proto
	go get -u github.com/gorilla/websocket
	go get -u github.com/lucasb-eyer/go-colorful
	go get -u github.com/pebbe/zmq4
	go get -u github.com/spf13/viper

proto:
	cd .. && protoc --go_out=blinken/pb pb.proto
