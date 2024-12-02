# Integrated circuit configuration

## TPS25730

| Pin    | Decoded value | Meaning  | Description              |
|--------|---------------|----------|--------------------------|
| ADCIN1 | 0             | 5V min   |                          |
| ADCIN2 | 1             | 9V max   | unable to set 5V maximum |
| ADCIN3 | 0             | 0.5A min | operating current        |
| ADCIN4 | 7             | 5A max   |                          |

* ADCIN1 =
  Table 8-3. Device Configuration using ADCIN1 and ADCIN2
  ADCIN1 DECODED
  VALUE (MIN
  VOLTAGE) (1)ADCIN2 DECODED
  VALUE (MAX
  VOLTAGE) (1)I2C ADDRESS
  0 (5V)7 (20V)0x20
  (1)
  1 (9V)7 (20V)0x21
  2 (12V)7 (20V)0x20
  3 (15V)7 (20V)0x21
  4 (20V)7 (20V)0x20
  0 (5V)5 (15V)0x20
  1 (9V)5 (15V)0x21
  2 (12V)5 (15V)0x20
  3 (15V)5 (15V)0x21
  0 (5V)3 (12V)0x20
  1 (9V)3 (12V)0x21
  2 (12V)3 (12V)0x20
  0 (5V)1 (9V)0x20
  1 (9V)1 (9V)0x21
  DEAD BATTERY CONFIGURATION
  AlwaysEnableSink: The device always enables the sink path
  regardless of the amount of current the attached source is
  offering. USB PD is disabled until configuration is loaded.
  See Section 8.3.6 for how to configure a given ADCINx decoded value.