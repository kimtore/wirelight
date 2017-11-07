# LEDServer by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
#
# This program drives LEDs from a Raspberry Pi Zero W using the rpi_ws281x
# library and Google Protobuf messages.

LED_CFLAGS = -Ivendor/rpi_ws281x
LED_LDFLAGS = -lws2811 -Lvendor/rpi_ws281x
LED_FLAGS = $(LED_CFLAGS) $(LED_LDFLAGS)

PROTOBUF_CFLAGS = $(shell pkg-config --cflags 'libprotobuf-c >= 1.0.0')
PROTOBUF_LDFLAGS = $(shell pkg-config --libs 'libprotobuf-c >= 1.0.0')
PROTOBUF_FLAGS = $(PROTOBUF_CFLAGS) $(PROTOBUF_LDFLAGS)

.PHONY: all proto grpc

all: led.o pb.pb-c.o server.o
	gcc $(LED_FLAGS) $(PROTOBUF_FLAGS) -o ledserver led.o pb.pb-c.o server.o

pb.pb-c.o: pb.pb-c.c
	gcc $(PROTOBUF_FLAGS) -Wall -c pb.pb-c.c

led.o: led.c
	gcc $(LED_FLAGS) -Wall -c led.c

server.o: server.c
	gcc -Wall -c server.c

proto:
	cd .. && protoc-c --c_out=ledserver pb.proto
