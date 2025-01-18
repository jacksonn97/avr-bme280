# avr-bme280

## Using
First you need to add this crate as dependence:
```toml
[dependencies]
avr-bme280 = { git = "https://github.com/jacksonn97/avr-bme280" }
```
Then you can use library just like:
```rust
use avr_bme280::i2c::BME280; // you can change i2c to spi as you want
let mut bme280 = BME280::init(&mut i2c, 0x76); // specify your i2c bus and address of sensor
                                               // for spi is spi bus and SS pin
let measure = bme280.get_measures(&mut i2c);
           // Return type is structure with temperature, humidity as pressure
```

### Connection SPI
| Board pin[^nano] | Sensor pin |
| ------- | --- |
| SCK 13  | SCL |
| MOSI 11 | SDA |
| MISO 12 | SDO |
| SS 10   | CSB |


You can you SPI definition like this:
```rust
let (mut spi, ss) = Spi::new(dp.SPI,
    pins.d13.into_output(),
    pins.d11.into_output(),
    pins.d12.into_pull_up_input(),
    pins.d10.into_output(),
    spi::Settings {
        data_order: arduino_hal::spi::DataOrder::MostSignificantFirst,
        clock: arduino_hal::spi::SerialClockRate::OscfOver4,
        mode: embedded_hal::spi::MODE_0, // you can also use MODE_3
    }
 );
```

### Connection I2C

| Board pin[^nano] | Sensor pin |
| --- | --- |
| SCL A5 | SCL |
| SDA A4 | SDA |

[^nano]: Specified pin numbers for Nano, watch pinout

## Roadmap
- [x] I2C support
- [x] SPI support
- [x] Docs
- [x] Datasheet [brief](https://github.com/jacksonn97/avr-bme280/blob/master/info.md)
