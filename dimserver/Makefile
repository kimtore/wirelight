# Dimserver by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
#
# This program drives a DAC which outputs 0-10V in order to dim the fluorescent tubes.

PROTOBUF_CFLAGS = $(shell pkg-config --cflags 'libprotobuf-c >= 1.0.0')
PROTOBUF_LDFLAGS = $(shell pkg-config --libs 'libprotobuf-c >= 1.0.0')
PROTOBUF_FLAGS = $(PROTOBUF_CFLAGS) $(PROTOBUF_LDFLAGS)

ZMQ_CFLAGS = $(shell pkg-config --cflags 'libzmq >= 3.0.0')
ZMQ_LDFLAGS = $(shell pkg-config --libs 'libzmq >= 3.0.0')
ZMQ_FLAGS = $(ZMQ_CFLAGS) $(ZMQ_LDFLAGS)

WIRINGPI_CFLAGS =
WIRINGPI_LDFLAGS = -lwiringPi
WIRINGPI_FLAGS = $(WIRINGPI_CFLAGS) $(WIRINGPI_LDFLAGS)

.PHONY: all proto grpc

all: dim.o
	gcc $(WIRINGPI_FLAGS) $(PROTOBUF_FLAGS) $(ZMQ_FLAGS) -o dimserver dim.o

pb.pb-c.o: pb.pb-c.c
	gcc $(PROTOBUF_FLAGS) -Wall -c pb.pb-c.c

dim.o: dim.c
	gcc $(WIRINGPI_FLAGS) -Wall -c dim.c

server.o: server.c
	gcc $(ZMQ_FLAGS) -Wall -c server.c

proto:
	cd .. && protoc-c --c_out=dimserver pb.proto
