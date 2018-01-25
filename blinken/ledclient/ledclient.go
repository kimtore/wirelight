package ledclient

import (
	"fmt"

	"github.com/pebbe/zmq4"
)

func Socket(address string) (*zmq4.Socket, error) {
	ctx, err := zmq4.NewContext()
	if err != nil {
		return nil, fmt.Errorf("while creating ZeroMQ context: %s\n", err)
	}

	sock, err := ctx.NewSocket(zmq4.PUB)
	if err != nil {
		return nil, fmt.Errorf("while creating ZeroMQ socket: %s\n", err)
	}

	err = sock.Connect(address)
	if err != nil {
		return nil, fmt.Errorf("while connecting to %s: %s\n", address, err)
	}

	return sock, nil
}
