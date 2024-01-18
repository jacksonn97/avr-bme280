
use embedded_hal::blocking::i2c::{WriteRead, Write};
use crate::*;

pub struct BME280 {
    address: u8,
    calibrationdata: CalibrationData,
    settings: Settings,
}

impl BME280 {
    #[inline]
    pub fn init(i2c: &mut hal_I2c, address: u8) -> Self {
        let pt_data = Self::read_calibration_data_pt(address, i2c);
        let h_data  = Self::read_calibration_data_h(address, i2c);

        let settings = Settings::default();

        Self {
            address,
            calibrationdata: CalibrationData::parse_calib_data(pt_data, h_data),
            settings,
        }
    }

    pub fn get_measures(&mut self, i2c: &mut hal_I2c) -> Measure {
        match self.settings.ctrl_meas & 0b00000011 {
            0b00000000 => (),
            0b00000011 => (),
            _ => self.trig_forced_measure(i2c)
        }
        Measure::parse(self.read_data(i2c), &mut self.calibrationdata)
    }

    fn read_data(&self, i2c: &mut hal_I2c) -> [u8; BME280_P_T_H_DATA_LEN] {
        let mut data: [u8; BME280_P_T_H_DATA_LEN] = [0; BME280_P_T_H_DATA_LEN];
        i2c.write_read(self.address, &[BME280_DATA_ADDR], &mut data).unwrap();
        data
    }

    // fn read_register(&self, i2c: &mut hal_I2c, register: u8) -> u8 {
    //     let mut data: [u8; 1] = [0];
    //     i2c.write_read(self.address, &[register], &mut data).unwrap();
    //     data[0]
    // }

    pub(crate) fn read_calibration_data_h(address: u8, i2c: &mut hal_I2c)
        -> [u8; BME280_H_CALIB_DATA_LEN]
    {
        let mut data = [0; BME280_H_CALIB_DATA_LEN];
        i2c.write_read(address, &[BME280_H_CALIB_DATA_ADDR], &mut data).unwrap();
        data
    }

    pub(crate) fn read_calibration_data_pt(address: u8, i2c: &mut hal_I2c)
        -> [u8; BME280_P_T_CALIB_DATA_LEN]
    {
        let mut data = [0; BME280_P_T_CALIB_DATA_LEN];
        i2c.write_read(address, &[BME280_P_T_CALIB_DATA_ADDR], &mut data).unwrap();
        data
    }

    fn write_register(&self, i2c: &mut hal_I2c, register: u8, data: u8) {
        i2c.write(self.address, &[register, data]).unwrap();
    }

    fn write_settings(&self, i2c: &mut hal_I2c) {
        self.write_register(i2c, CONFIG, self.settings.config);
        self.write_register(i2c, CTRL_MEAS, self.settings.ctrl_meas);
        self.write_register(i2c, CTRL_HUM, self.settings.ctrl_hum);
    }

    pub fn update_settings(&mut self, i2c: &mut hal_I2c, settings: Settings) {
        self.settings = settings;
        self.write_settings(i2c)
    }

    pub fn reset_sensor(&self, i2c: &mut hal_I2c) {
        self.write_register(i2c, RESET, RESET_CODE)
    }

    fn trig_forced_measure(&self, i2c: &mut hal_I2c) {
        i2c.write(self.address, &[CTRL_MEAS, self.settings.ctrl_meas]).unwrap()
    }
}

