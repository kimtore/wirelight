FROM golang:alpine AS build
RUN apk --no-cache add git gcc
RUN go get github.com/dorkowscy/lyslix
RUN go get github.com/eclipse/paho.mqtt.golang
RUN go get github.com/spf13/viper
RUN go get github.com/lucasb-eyer/go-colorful
ADD . /go/src/github.com/ambientsound/wirelight/labpanel/colorizer
WORKDIR /go/src/github.com/ambientsound/wirelight/labpanel/colorizer
ENV GOOS=linux
ENV CGO_ENABLED=0
ENV GO111MODULE=on
RUN go build -a -installsuffix cgo -o /colorizer

FROM alpine
COPY --from=build /colorizer /colorizer
CMD ["/colorizer"]
