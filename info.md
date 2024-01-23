# BME280

## Brief information
- Measurements temperature, humidity, presurement
- Operating ranges:
   - -40..85C
   - 0..100% rel humidity
   - 300..1100 hPa
- Voltage: 1.7 to 3.6 V
- Support 3 power modes:
   - sleep mode
   - normal mode
   - forced mode

### Sensor modes

#### Sleep mode
Lowest power, no operations, active after startup by default.
All registers are accessible; Chip-ID and compensation coefficients can be read.

#### Forces mode
Perform one measurement, store results and return in sleep mode.
Using forced mode is recommended for applications which require low sampling rate or host-based synchronization.

Cycle time = rate of force mode

#### Normal mode
Infinity loop of measurements with inactivity(standby) period.
Standby can be determined between 0.5 and 1000ms

Cycle time = time of measurements + standby time

#### Recomended types of measurements

##### Weather monitoring (default)
- forced mode
- pressure x1, temperature x1, humidity x1
- 1 s/m
- filter off
##### Humidity sending
- forced/nomal mode
- pressure x0, temperature x1, humidity x1
- 1 s/s
- filter off
##### Indoor monitoring
- normal mode, standby = 0.5ms
- pressure x16, temperature x2, humidity x1
- filter coefficient 16

### I2C bus
- `0x76` - default address
- `0x77` - secondary address
