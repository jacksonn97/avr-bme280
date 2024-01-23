# avr-bme280

Library based on [avr-hal](https://github.com/Rahix/avr-hal)
=

## Using
You can use library just like:
```rust
use bme280::i2c::BME280; // you can change i2c to spi as you want
let mut bme280 = BME280::init(&mut i2c, 0x76); // specify your i2c bus and address of sensor
                                               // for spi is spi bus and SS pin
let mesrure = bme280.get_measures(&mut i2c);
           // Return type is structure with temperature, humidity as pressure
```


### Roadmap
- [x] I2C support
- [x] SPI support
- [ ] Error processing?
- [ ] Docs
- [ ] Datasheet brief

### :warning: Important info :warning:
If you can test library on some board from list contact with me(tested is marked):
- [ ] Arduino Leonardo
- [ ] Arduino Mega 2560
- [ ] Arduino Mega 1280
- [x] Arduino Nano
- [ ] Arduino Uno
