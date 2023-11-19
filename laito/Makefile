all:
	pio run

build:
	pio run -t buildfs

upload:
	pio run -t upload

monitor:
	pio device monitor -b 115200

test: upload monitor
