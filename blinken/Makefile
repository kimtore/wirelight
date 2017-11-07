# Blinken by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
#
# This program sends Google Protobuf messages to LEDServer.

.PHONY: all proto

all:
	go build

proto:
	cd .. && protoc --go_out=blinken/pb pb.proto
