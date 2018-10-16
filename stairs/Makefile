all:
	pio run

upload:
	pio run -t upload

test: upload
	pio device monitor -b 115200
