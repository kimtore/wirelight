FROM golang:alpine AS build
RUN apk --no-cache add git pkgconf zeromq-dev musl-dev gcc alpine-sdk libsodium-dev
RUN go get github.com/pebbe/zmq4
RUN go get github.com/eclipse/paho.mqtt.golang
RUN go get github.com/gorilla/websocket
RUN go get github.com/spf13/viper
RUN go get github.com/golang/protobuf/proto
RUN go get github.com/lucasb-eyer/go-colorful
ADD . /go/src/github.com/ambientsound/wirelight/blinken
WORKDIR /go/src/github.com/ambientsound/wirelight/blinken
RUN CGO_LDFLAGS="$CGO_LDFLAGS -lstdc++ -lm -lsodium" \
    CGO_ENABLED=1 \
    GOOS=linux \
    go build -o /blinken -a --ldflags '-extldflags "-static" -v'

FROM alpine
RUN apk add --no-cache ca-certificates
COPY --from=build /blinken /blinken
EXPOSE 8081
CMD ["/blinken"]
