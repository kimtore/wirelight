ARDUINO_PATH = /opt/arduino
SKETCHBOOK	 = $(HOME)/Arduino
ESPTOOL		 = $(SKETCHBOOK)/hardware/esp8266/esp8266/tools/esptool/esptool
SKETCH		 = $(notdir $(CURDIR)).ino
TARGET_DIR	 = $(CURDIR)/build-esp8266
MONITOR_PORT = /dev/ttyUSB0


all:
	@ mkdir -p $(TARGET_DIR)

	$(ARDUINO_PATH)/arduino-builder -compile -logger=machine \
	-hardware "$(ARDUINO_PATH)/hardware" \
	-tools "$(ARDUINO_PATH)/tools-builder" \
	-tools "$(ARDUINO_PATH)/hardware/tools/avr" \
	-built-in-libraries "$(ARDUINO_PATH)/libraries" \
	-libraries "$(SKETCHBOOK)/libraries" \
	-fqbn=esp8266com:esp8266:nodemcuv2:CpuFrequency=80,UploadSpeed=115200,FlashSize=4M3M \
	-ide-version=10611 \
	-build-path "$(TARGET_DIR)" \
	-warnings=none \
	-prefs=build.warn_data_percentage=75 \
	-verbose "$(SKETCH)"

flash:
	$(ESPTOOL) -v -cd nodemcu -cb 115200 -cp $(MONITOR_PORT) -ca 0x00000 -cf $(TARGET_DIR)/$(SKETCH).bin

upload: all flash

clean:
	rm -rf $(TARGET_DIR)

monitor:
	screen $(MONITOR_PORT) 9600
