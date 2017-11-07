LED_CFLAGS = -Ivendor/rpi_ws281x
LED_LDFLAGS = -lws2811 -Lvendor/rpi_ws281x
LED_FLAGS = $(LED_CFLAGS) $(LED_LDFLAGS)

PROTOBUF_CFLAGS = $(shell pkg-config --cflags protobuf)
PROTOBUF_LDFLAGS = $(shell pkg-config --libs protobuf)
PROTOBUF_FLAGS = $(PROTOBUF_CFLAGS) $(PROTOBUF_LDFLAGS)

.PHONY: all led proto

all: led.o pb.pb.o
	g++ $(LED_FLAGS) $(PROTOBUF_FLAGS) -o ledserver led.o pb.pb.o

main.o: main.cc
	g++ $(PROTOBUF_FLAGS) -Wall -c main.cc

pb.grpc.pb.o: pb.grpc.pb.cc
	g++ $(PROTOBUF_FLAGS) -Wall -c pb.grpc.pb.cc

pb.pb.o: pb.pb.cc
	g++ $(PROTOBUF_FLAGS) -Wall -c pb.pb.cc

led.o: led.cc
	gcc $(LED_FLAGS) -Wall -c led.cc

proto:
	cd .. && protoc --cpp_out=ledserver pb.proto

grpc:
	cd .. && protoc --plugin=protoc-gen-grpc=/usr/local/bin/grpc_cpp_plugin --grpc_out=ledserver pb.proto
