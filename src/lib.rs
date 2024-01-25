#![no_std]
#![warn(missing_docs)]

//! Library for BME280
//! Library based on [avr-hal](https://github.com/Rahix/avr-hal)
//!
//! ## Using
//! Then you can use library just like:
//! ```rust
//! use avr_bme280::i2c::BME280; // you can change i2c to spi as you want
//! let mut bme280 = BME280::init(&mut i2c, 0x76); // specify your i2c bus and address of sensor
//!                                                // for spi is spi bus and SS pin
//! let measure = bme280.get_measures(&mut i2c);
//!            // Return type is structure with temperature, humidity as pressure
//! ```

pub(crate) use arduino_hal::{I2c as hal_I2c, Spi as hal_Spi};

// re-exports


/// Implementation I2C bus
pub mod i2c;

/// Implementation SPI bus
pub mod spi;

macro_rules! set_const {
    ($name:ident, $val:literal) => {
        const $name: u8 = $val;
    };
    ($name:ident, $val:literal, $type:ty) => {
        const $name: $type = $val;
    };
}

macro_rules! concat_bytes {
    ($msb:expr, $lsb:expr) => {
        (($msb as u16) << 8) | ($lsb as u16)
    };
}

// register names
set_const!(CONFIG,    0xF5);
set_const!(CTRL_MEAS, 0xF4);
set_const!(CTRL_HUM,  0xF2);

set_const!(RESET,      0xE0);
set_const!(RESET_CODE, 0xB6);

set_const!(BME280_H_CALIB_DATA_ADDR, 0xE1);
set_const!(BME280_P_T_CALIB_DATA_ADDR, 0x88);
set_const!(BME280_H_CALIB_DATA_LEN, 7, usize);
set_const!(BME280_P_T_CALIB_DATA_LEN, 26, usize);

set_const!(BME280_DATA_ADDR, 0xF7);
set_const!(BME280_P_T_H_DATA_LEN, 8, usize);


/// Out type for user
pub struct Measure {
    /// ?? i must explain this ??
    pub temperature: f32,
    /// https://imgur.com/a/NOEPqa3
    pub humidity: f32,
    /// yeah, this pressure
    pub pressure: f32,
}

/// BME280 settings
///
/// By default used settings for the weather measures
///  - pressure x 1, temperature x 1, humidity x 1
///  - forced mode
///  - filter off
///
///  Read the datasheet
pub struct Settings {
    /// t_sb[2:0], filter[2:0], \<not used\>[0], spi3w_en[0]
    pub config: u8,
    /// osrs_t[2:0], osrs_p[2:0], mode[1:0]
    pub ctrl_meas: u8,
    /// \<not used\>[5:0], osrs_h[2:0]
    pub ctrl_hum: u8,
}

pub(crate) struct CalibrationData {
    dig_t1: u16,
    dig_t2: i16,
    dig_t3: i16,
    dig_p1: u16,
    dig_p2: i16,
    dig_p3: i16,
    dig_p4: i16,
    dig_p5: i16,
    dig_p6: i16,
    dig_p7: i16,
    dig_p8: i16,
    dig_p9: i16,
    dig_h1: u8,
    dig_h2: i16,
    dig_h3: u8,
    dig_h4: i16,
    dig_h5: i16,
    dig_h6: i8,
    t_fine: i32,
}

impl CalibrationData {
    #[inline]
    fn fix_temp(&mut self, adc_t: i32) -> i32 {
        let var1: i32 = ((((adc_t>>3) - ((self.dig_t1 as i32)<<1))) * ((self.dig_t2 as i32))) >> 11;
        let var2: i32 = (((((adc_t>>4) - ((self.dig_t1 as i32))) * ((adc_t>>4) - ((self.dig_t1 as i32)))) >> 12) * ((self.dig_t3 as i32))) >> 14;
        self.t_fine = var1 + var2;
        (self.t_fine * 5 + 128) >> 8
    }

    #[inline]
    fn fix_pres(&self, adc_p: i32) -> u32 {
        let mut var1: i64 = (self.t_fine as i64) - 128000;
        let mut var2: i64 = var1 * var1 * (self.dig_p6 as i64);
        let mut p: i64 = 1048576 - adc_p as i64;

        var2 = var2 + ((var1*(self.dig_p5 as i64))<<17);
        var2 = var2 + (((self.dig_p4 as i64))<<35);
        var1 = ((var1 * var1 * (self.dig_p3 as i64))>>8) + ((var1 * (self.dig_p2 as i64))<<12);
        var1 = ((((1)<<47)+var1))*((self.dig_p1 as i64))>>33;
        if var1 == 0 {
            return 0; // avoid exception caused by division by zero
        }
        p = (((p<<31)-var2)*3125)/var1;
        var1 = (((self.dig_p9 as i64)) * (p>>13) * (p>>13)) >> 25;
        var2 = (((self.dig_p8 as i64)) * p) >> 19;
        (((p + var1 + var2) >> 8) + (((self.dig_p7 as i64))<<4)) as u32
    }

    #[inline]
    fn fix_humid(&self, adc_h: i32) -> u32 {
        let mut var = self.t_fine - (76800);
        var = ((((adc_h << 14) - (((self.dig_h4 as i32)) << 20) - (((self.dig_h5 as i32)) * var)) + (16384)) >> 15) * (((((((var * ((self.dig_h6 as i32))) >> 10) * (((var * ((self.dig_h3 as i32))) >> 11) + (32768))) >> 10) + (2097152)) * ((self.dig_h2 as i32)) + 8192) >> 14);
        var = var - (((((var >> 15) * (var >> 15)) >> 7) * ((self.dig_h1 as i32))) >> 4);
        var = var.clamp(0, 419430400);
        (var>>12) as u32
    }

    fn parse_calib_data(pt_data: [u8; BME280_P_T_CALIB_DATA_LEN], h_data: [u8; BME280_H_CALIB_DATA_LEN]) -> CalibrationData {
        let dig_t1 = concat_bytes!(pt_data[1], pt_data[0]);
        let dig_t2 = concat_bytes!(pt_data[3], pt_data[2]) as i16;
        let dig_t3 = concat_bytes!(pt_data[5], pt_data[4]) as i16;
        let dig_p1 = concat_bytes!(pt_data[7], pt_data[6]);
        let dig_p2 = concat_bytes!(pt_data[9], pt_data[8]) as i16;
        let dig_p3 = concat_bytes!(pt_data[11], pt_data[10]) as i16;
        let dig_p4 = concat_bytes!(pt_data[13], pt_data[12]) as i16;
        let dig_p5 = concat_bytes!(pt_data[15], pt_data[14]) as i16;
        let dig_p6 = concat_bytes!(pt_data[17], pt_data[16]) as i16;
        let dig_p7 = concat_bytes!(pt_data[19], pt_data[18]) as i16;
        let dig_p8 = concat_bytes!(pt_data[21], pt_data[20]) as i16;
        let dig_p9 = concat_bytes!(pt_data[23], pt_data[22]) as i16;
        let dig_h1 = pt_data[25];
        let dig_h2 = concat_bytes!(h_data[1], h_data[0]) as i16;
        let dig_h3 = h_data[2];
        let dig_h4 = (h_data[3] as i8 as i16 * 16) | ((h_data[4] as i8 as i16) & 0x0F);
        let dig_h5 = (h_data[5] as i8 as i16 * 16) | (((h_data[4] as i8 as i16) & 0xF0) >> 4);
        let dig_h6 = h_data[6] as i8;

        CalibrationData {
            dig_t1,
            dig_t2,
            dig_t3,
            dig_p1,
            dig_p2,
            dig_p3,
            dig_p4,
            dig_p5,
            dig_p6,
            dig_p7,
            dig_p8,
            dig_p9,
            dig_h1,
            dig_h2,
            dig_h3,
            dig_h4,
            dig_h5,
            dig_h6,
            t_fine: 0,
        }
    }

}

impl Measure {
    pub(crate) fn parse(
        data: [u8; BME280_P_T_H_DATA_LEN],
        calibration: &mut CalibrationData,
    ) -> Self {
        let data_msb = (data[0] as u32) << 12;
        let data_lsb = (data[1] as u32) << 4;
        let data_xlsb = (data[2] as u32) >> 4;
        let pressure = data_msb | data_lsb | data_xlsb;

        let data_msb = (data[3] as u32) << 12;
        let data_lsb = (data[4] as u32) << 4;
        let data_xlsb = (data[5] as u32) >> 4;
        let temperature = data_msb | data_lsb | data_xlsb;

        let data_msb = (data[6] as u32) << 8;
        let data_lsb = data[7] as u32;
        let humidity = data_msb | data_lsb;

        Measure {
            temperature: CalibrationData::fix_temp(calibration, temperature as i32) as f32 / 100.0,
            pressure: CalibrationData::fix_pres(calibration, pressure as i32) as f32 / 256.0,
            humidity: CalibrationData::fix_humid(calibration, humidity as i32) as f32 / 1024.0,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self { config: 0b00000000, ctrl_meas: 0b00100110, ctrl_hum: 0b00000001 }
    }
}

