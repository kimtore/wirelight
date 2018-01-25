// package ws provides a websocket interface for sending real-time color updates.
package ws

import (
	"encoding/json"
	"log"
	"net/http"

	"github.com/gorilla/websocket"
	colorful "github.com/lucasb-eyer/go-colorful"
)

type State struct {
	Effect string
	H      uint16
	S      uint16
	V      uint16
}

func checkOrigin(r *http.Request) bool {
	return true
}

var upgrader = websocket.Upgrader{
	ReadBufferSize:  16384,
	WriteBufferSize: 16384,
	CheckOrigin:     checkOrigin,
}

func MakeColor(m State) colorful.Color {
	return colorful.Hsv(
		float64(m.H)/65535.0*360,
		float64(m.S)/65535.0,
		float64(m.V)/65535.0,
	)
}

func Serve(addr, path string, messages chan State) {
	http.HandleFunc(path, func(w http.ResponseWriter, r *http.Request) {

		// Upgrade the connection to a websocket connection.
		conn, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			log.Println(err)
			return
		}

		// Send the initial server state.
		// FIXME
		err = conn.WriteMessage(websocket.TextMessage, []byte("foobar"))
		if err != nil {
			log.Printf("while sending initial message: %s\n", err)
			return
		}

		for {
			// Receive websocket payload.
			_, payload, err := conn.ReadMessage()
			if err != nil {
				log.Println(err)
				return
			}

			// Decode JSON.
			m := State{}
			err = json.Unmarshal(payload, &m)
			if err != nil {
				log.Printf("while unmarshalling: %s\n", err)
				return
			}

			// Pass message on to the effect engine.
			messages <- m
		}
	})

	http.ListenAndServe(addr, nil)
}
