--dofile("config.lua")
ws2812.init(ws2812.MODE_SINGLE)
leds = ws2812.newBuffer(60, 3)
leds:fill(0, 0, 0)

local on = tmr.create()

function init_blink_led()
    leds:set(1, 0, 0, 80)
    ws2812.write(leds)
    tmr.create():alarm(100, tmr.ALARM_SINGLE, function()
        leds:set(1, 0, 0, 0)
        ws2812.write(leds)
    end)
end

on:alarm(500, tmr.ALARM_AUTO, init_blink_led)

tmr.create():alarm(2000, tmr.ALARM_SINGLE, function()
    on:unregister()
    dofile("config.lua")
    dofile("start.lua")
end)

init_blink_led()
