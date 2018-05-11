-- initialize leds
leds = ws2812.newBuffer(NUM_LEDS, BYTES_PER_LED)
leds_color = string.char(20, 20, 20)

-- init mqtt client with logins, keepalive timer 120sec
m = mqtt.Client(HOSTNAME, 120, MQTT_USER, MQTT_PASSWORD)

-- system status cache
system_status = {wifi="wait", mqtt="wait", comm="wait"}

function led_render()
    ws2812.write(leds)
end

function led_status_color(status)
    if     status == "error"    then return string.char(0, 255, 0)
    elseif status == "ok"       then return string.char(255, 0, 0)
    elseif status == "wait"     then return string.char(255, 255, 0)
    else                             return string.char(0, 0, 255)
    end
end

function led_effect_status()
    leds:fill(0, 0, 0)
    leds:set(1, led_status_color(system_status.wifi))
    leds:set(2, led_status_color(system_status.mqtt))
    leds:set(3, led_status_color(system_status.comm))
end

function led_effect_static()
    for led = 1, NUM_LEDS do
        leds:set(led, leds_color)
        --leds:set(led, 0, 30, 0)
    end
end

-- LED effect table
led_effect = "status"
led_effects = {
    status=led_effect_status,
    static=led_effect_static
}

function led_run_effect()
    led_effects[led_effect]()
    led_render()
end

function handle_wifi_connect()
    system_status.wifi = "ok"
end

function handle_wifi_ip()
    print("wifi: got IP, connecting to mqtt")
    do_mqtt_connect()
end

function wifi_init()
    wifi.setmode(wifi.STATION)
    cfg = {}
    cfg.ssid = WIFI_SSID
    cfg.pwd = WIFI_PASSWORD
    cfg.save = false
    cfg.connected_cb = handle_wifi_connect
    cfg.got_ip_cb = handle_wifi_ip
    wifi.sta.sethostname(HOSTNAME)
    wifi.sta.config(cfg)
end

function handle_mqtt_error(client, reason) 
	print("mqtt: failed to connect, reason: " .. reason)
    system_status.mqtt = "error"
    client:close()
	tmr.create():alarm(10 * 1000, tmr.ALARM_SINGLE, do_mqtt_connect)
end

function handle_mqtt_connect(client)
	print("mqtt: connected")
    system_status.mqtt = "ok"
    led_effect = "static"
	client:subscribe(MQTT_TOPIC, 0, function(client) print("mqtt: subscribed to topic") end)
end

-- Create effect runner
effect = tmr.create()
effect:alarm(50, tmr.ALARM_AUTO, led_run_effect)

function handle_mqtt_message(client, topic, data) 
    if data == nil then
        return
    end
    print("mqtt: receive: " .. data)
    system_status.comm = "ok"
    t = sjson.decode(data)
    if t.state ~= nil then
        if t.state == "OFF" then
            effect:stop()
            leds:fill(0, 0, 0)
            led_render()
        else
            effect:start()
        end
    end
    if t.color ~= nil then
        leds_color = string.char(t.color.g, t.color.r, t.color.b)
    end
    if t.effect ~= nil then
        if led_effects[t.effect] ~= nil then
            led_effect = t.effect
        else
            led_effect = "status"
            system_status.comm = "error"
        end
    end
end

function do_mqtt_connect()
    system_status.mqtt = "wait"
    m:connect("10.42.0.2", 8883, 1, 0, handle_mqtt_connect, handle_mqtt_error)
end

m:on("message", handle_mqtt_message)

-- connect to network
wifi_init()
