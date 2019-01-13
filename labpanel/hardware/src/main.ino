// Configuration
#include "config.h"

#define SERIAL_SPEED    115200  // Serial baud rate.
#define HARDWARE_SPI    D8      // "CS" port of the SPI interface. Corresponds to HCS, or GPIO15.
#define ANALOG_INPUTS   4       // Number of analog pins connected to the MCP3008 chip.
#define LOOP_DELAY      50      // Time between potentiometer scans.

// Third party libraries
#include <Adafruit_MCP3008.h>
#include <ESP8266WiFi.h>
#include <PubSubClient.h>

Adafruit_MCP3008 adc;
WiFiClient wifi;
PubSubClient mqtt;

uint16 results[ANALOG_INPUTS];
uint16 prev_results[ANALOG_INPUTS];

void setup() {
#ifdef SERIAL_DEBUG
    Serial.begin(SERIAL_SPEED);
    while (!Serial);
#endif

    adc.begin(HARDWARE_SPI);

    WiFi.begin(WIFI_SSID, WIFI_PASSWORD);

    mqtt.setClient(wifi);
    mqtt.setServer(MQTT_SERVER_IP, MQTT_SERVER_PORT);
}

void mqtt_connect() {
#ifdef SERIAL_DEBUG
    Serial.println("Attempting MQTT connection...");
#endif
    if (mqtt.connect(MQTT_CLIENT_ID, MQTT_USER, MQTT_PASSWORD)) {
#ifdef SERIAL_DEBUG
        Serial.println("MQTT connected");
#endif
        send_results();
    } else {
#ifdef SERIAL_DEBUG
        Serial.print("ERROR: failed, rc=");
        Serial.print(mqtt.state());
        Serial.println("DEBUG: try again in 5 seconds");
        delay(5000);
#endif
    }
}

bool changed() {
    int diff;
    for (int i = 0; i < ANALOG_INPUTS; i++) {
        diff = results[i] - prev_results[i];
        // disregard changes of one due to jitter
        if (diff < -1 || diff > 1) {
            return true;
        }
    }
    return false;
}

#ifdef SERIAL_DEBUG
void print_results() {
    for (int i = 0; i < ANALOG_INPUTS; i++) {
        Serial.print(results[i]);
        Serial.print("\t");
    }
    Serial.print("\n");
}
#endif

void send_results() {
    char buf[32];
    char *dest = &buf[0];

    if (!mqtt.connected()) {
        return;
    }

    for (int i = 0; i < ANALOG_INPUTS; i++) {
        dest += sprintf(dest, "%d ", results[i]);
    }
    dest[0] = '\0';

    mqtt.publish(MQTT_TOPIC, buf);
}

void loop() {
    delay(LOOP_DELAY);

    if (WiFi.status() != WL_CONNECTED) {
        return;
    }

    if (!mqtt.connected()) {
        mqtt_connect();
        return;
    }

    mqtt.loop();

    for (int i = 0; i < ANALOG_INPUTS; i++) {
        results[i] = adc.readADC(i);
    }

    if (changed()) {
#ifdef SERIAL_DEBUG
        print_results();
#endif
        send_results();
        memcpy(&prev_results, &results, sizeof(results));
    }

}
