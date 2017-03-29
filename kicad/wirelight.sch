EESchema Schematic File Version 2
LIBS:power
LIBS:device
LIBS:transistors
LIBS:conn
LIBS:linear
LIBS:regul
LIBS:74xx
LIBS:cmos4000
LIBS:adc-dac
LIBS:memory
LIBS:xilinx
LIBS:microcontrollers
LIBS:dsp
LIBS:microchip
LIBS:analog_switches
LIBS:motorola
LIBS:texas
LIBS:intel
LIBS:audio
LIBS:interface
LIBS:digital-audio
LIBS:philips
LIBS:display
LIBS:cypress
LIBS:siliconi
LIBS:opto
LIBS:atmel
LIBS:contrib
LIBS:valves
LIBS:wirelight
LIBS:wirelight-cache
EELAYER 25 0
EELAYER END
$Descr A4 11693 8268
encoding utf-8
Sheet 1 1
Title "Wirelight"
Date "2017-03-28"
Rev "1.0"
Comp "ambientsound"
Comment1 ""
Comment2 ""
Comment3 ""
Comment4 ""
$EndDescr
$Comp
L R R?
U 1 1 58DAAA69
P 7250 3650
F 0 "R?" V 7330 3650 50  0000 C CNN
F 1 "470" V 7250 3650 50  0000 C CNN
F 2 "" V 7180 3650 50  0000 C CNN
F 3 "" H 7250 3650 50  0000 C CNN
	1    7250 3650
	0    -1   -1   0   
$EndComp
$Comp
L ARDUINO_NANO U?
U 1 1 58DAB9CD
P 5350 3550
F 0 "U?" H 5350 3450 50  0000 C CNN
F 1 "ARDUINO_NANO" H 5350 3650 50  0000 C CNN
F 2 "MODULE" H 5350 3550 50  0001 C CNN
F 3 "DOCUMENTATION" H 5350 3550 50  0001 C CNN
	1    5350 3550
	1    0    0    -1  
$EndComp
$Comp
L GND #PWR?
U 1 1 58DABB8C
P 1850 4750
F 0 "#PWR?" H 1850 4500 50  0001 C CNN
F 1 "GND" H 1850 4600 50  0000 C CNN
F 2 "" H 1850 4750 50  0000 C CNN
F 3 "" H 1850 4750 50  0000 C CNN
	1    1850 4750
	1    0    0    -1  
$EndComp
$Comp
L +5V #PWR?
U 1 1 58DABBA6
P 1850 2300
F 0 "#PWR?" H 1850 2150 50  0001 C CNN
F 1 "+5V" H 1850 2440 50  0000 C CNN
F 2 "" H 1850 2300 50  0000 C CNN
F 3 "" H 1850 2300 50  0000 C CNN
	1    1850 2300
	1    0    0    -1  
$EndComp
Wire Bus Line
	2050 2300 9300 2300
$Comp
L CP C?
U 1 1 58DABD49
P 3600 3550
F 0 "C?" H 3625 3650 50  0000 L CNN
F 1 "CP" H 3625 3450 50  0000 L CNN
F 2 "" H 3638 3400 50  0000 C CNN
F 3 "" H 3600 3550 50  0000 C CNN
	1    3600 3550
	1    0    0    -1  
$EndComp
$Comp
L BARREL_JACK CON?
U 1 1 58DABEF4
P 2550 3550
F 0 "CON?" H 2550 3800 50  0000 C CNN
F 1 "BARREL_JACK" H 2550 3350 50  0000 C CNN
F 2 "" H 2550 3550 50  0000 C CNN
F 3 "" H 2550 3550 50  0000 C CNN
	1    2550 3550
	1    0    0    -1  
$EndComp
Wire Wire Line
	3600 2300 3600 3400
Wire Wire Line
	3600 3700 3600 4750
Wire Wire Line
	3050 3450 2850 3450
Wire Wire Line
	3050 2300 3050 3450
Wire Wire Line
	2850 3550 3050 3550
Wire Wire Line
	3050 3550 3050 4750
Wire Wire Line
	2850 3650 3050 3650
Connection ~ 3050 3650
Wire Wire Line
	4600 4150 4150 4150
Wire Wire Line
	4150 4150 4150 4750
$Comp
L NEOPIXELS P?
U 1 1 58DC1FF2
P 8100 3650
F 0 "P?" H 8100 3850 50  0000 C CNN
F 1 "NEOPIXELS" V 8450 3650 50  0000 C CNN
F 2 "" H 8100 3650 50  0000 C CNN
F 3 "" H 8100 3650 50  0000 C CNN
	1    8100 3650
	1    0    0    -1  
$EndComp
$Comp
L ESP8266-REV7 P?
U 1 1 58DC2264
P 7100 2950
F 0 "P?" H 7100 3150 50  0000 C CNN
F 1 "ESP8266-REV7" V 7450 2950 50  0000 C CNN
F 2 "" H 7100 2950 50  0000 C CNN
F 3 "" H 7100 2950 50  0000 C CNN
	1    7100 2950
	1    0    0    -1  
$EndComp
Wire Wire Line
	6950 3050 6650 3050
Wire Wire Line
	6650 3050 6650 2850
Wire Wire Line
	6650 2850 6100 2850
Wire Wire Line
	6100 2950 6550 2950
Wire Wire Line
	6550 2950 6550 3150
Wire Wire Line
	6550 3150 6950 3150
Wire Wire Line
	6950 2850 6750 2850
Wire Wire Line
	6750 2850 6750 4750
Wire Wire Line
	6950 2950 6850 2950
Wire Wire Line
	6850 2950 6850 2300
Wire Wire Line
	7900 3550 7750 3550
Wire Wire Line
	7750 3550 7750 2300
Wire Wire Line
	7900 3650 7400 3650
Wire Wire Line
	6100 3650 7100 3650
Wire Wire Line
	7900 3750 7750 3750
Wire Wire Line
	7750 3750 7750 4750
Connection ~ 7750 4750
Connection ~ 6750 4750
Connection ~ 4150 4750
Connection ~ 3050 2300
Connection ~ 3600 2300
Connection ~ 3600 4750
Connection ~ 3050 4750
Connection ~ 1850 2300
Connection ~ 1850 4750
Wire Bus Line
	2050 4750 9350 4750
Wire Wire Line
	6100 3150 6250 3150
Wire Wire Line
	6250 3150 6250 4750
Connection ~ 6250 4750
Connection ~ 6850 2300
Connection ~ 7750 2300
$EndSCHEMATC