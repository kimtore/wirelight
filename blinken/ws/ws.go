// package ws provides a websocket interface for sending real-time color updates.
package ws

import (
	"encoding/json"
	"log"
	"net/http"

	"github.com/gorilla/websocket"
	colorful "github.com/lucasb-eyer/go-colorful"
)

type Message struct {
	Effect    string
	Hue       uint16
	Chroma    uint16
	Luminance uint16
}

func checkOrigin(r *http.Request) bool {
	return true
}

var upgrader = websocket.Upgrader{
	ReadBufferSize:  16384,
	WriteBufferSize: 16384,
	CheckOrigin:     checkOrigin,
}

func MakeColor(m Message) colorful.Color {
	return colorful.Hcl(
		float64(m.Hue)/65535.0*360,
		float64(m.Chroma)/65535.0,
		float64(m.Luminance)/65535.0,
	)
}

func Serve(addr, path string, messages chan Message) {
	http.HandleFunc(path, func(w http.ResponseWriter, r *http.Request) {
		conn, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			log.Println(err)
			return
		}

		err = conn.WriteMessage(websocket.TextMessage, []byte("foobar"))
		if err != nil {
			log.Printf("while sending initial message: %s\n", err)
			return
		}

		for {
			_, payload, err := conn.ReadMessage()
			if err != nil {
				log.Println(err)
				return
			}
			m := Message{}
			err = json.Unmarshal(payload, &m)
			if err != nil {
				log.Printf("while unmarshalling: %s\n", err)
				return
			}
			messages <- m
		}
	})

	http.ListenAndServe(addr, nil)
}
