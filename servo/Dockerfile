FROM golang:1.14-alpine AS build
ADD . /src
WORKDIR /src
RUN go build -o /servo cmd/servo/*.go

FROM alpine
RUN apk add --no-cache ca-certificates
COPY --from=build /servo /servo
CMD ["/servo"]
