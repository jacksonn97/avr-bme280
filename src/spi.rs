
use embedded_hal::{blocking::spi::{Transfer, Write}, digital::v2::OutputPin};
use crate::*;

pub struct BME280<T: OutputPin> {
    ss: T,
    calibrationdata: CalibrationData,
    settings: Settings,
}

impl<T: OutputPin> BME280<T> {
    pub fn init(combined: (&mut hal_Spi, T)) -> Self {
        let (mut spi, mut ss) = combined;
        let pt_data = Self::read_calibration_data_pt(&mut spi, &mut ss);
        let h_data  = Self::read_calibration_data_h(&mut spi, &mut ss);

        let settings = Settings::default();

        Self {
            ss,
            calibrationdata: CalibrationData::parse_calib_data(pt_data, h_data),
            settings,
        }
    }

    pub fn get_measures(&mut self, spi: &mut hal_Spi) -> Measure {
        match self.settings.ctrl_meas & 0b00000011 {
            0b00000000 => (),
            0b00000011 => (),
            _ => self.trig_forced_measure(spi)
        }
        Measure::parse(self.read_data(spi), &mut self.calibrationdata)
    }

    fn read_data(&mut self, spi: &mut hal_Spi) -> [u8; BME280_P_T_H_DATA_LEN] {
        let mut data: [u8; BME280_P_T_H_DATA_LEN] = [0; BME280_P_T_H_DATA_LEN];

        self.ss.set_low();

        spi.write(&[BME280_DATA_ADDR | 0x80]).unwrap();
        spi.transfer(&mut data).unwrap();

        self.ss.set_high();
        data
    }

    fn read_calibration_data_pt(spi: &mut hal_Spi, ss: &mut T) -> [u8; BME280_P_T_CALIB_DATA_LEN] {
        let mut data: [u8; BME280_P_T_CALIB_DATA_LEN] = [0; BME280_P_T_CALIB_DATA_LEN];
        ss.set_low();

        spi.write(&[BME280_P_T_CALIB_DATA_ADDR | 0x80]).unwrap();
        spi.transfer(&mut data).unwrap();

        ss.set_high();
        data
    }

    fn read_calibration_data_h(spi: &mut hal_Spi, ss: &mut T) -> [u8; BME280_H_CALIB_DATA_LEN] {
        let mut data: [u8; BME280_H_CALIB_DATA_LEN] = [0; BME280_H_CALIB_DATA_LEN];

        ss.set_low();

        spi.write(&[BME280_H_CALIB_DATA_ADDR | 0x80]).unwrap();
        spi.transfer(&mut data).unwrap();

        ss.set_high();
        data
    }

    fn write_register(&mut self, spi: &mut hal_Spi, register: u8, data: u8) {

        let data = [register & 0x7f, data];
        self.ss.set_low();

        spi.write(&data).unwrap();

        self.ss.set_high();
    }

    #[inline]
    fn write_settings(&mut self, spi: &mut hal_Spi) {
        self.write_register(spi, CONFIG, self.settings.config);
        self.write_register(spi, CTRL_MEAS, self.settings.ctrl_meas);
        self.write_register(spi, CTRL_HUM, self.settings.ctrl_hum);
    }

    pub fn update_settings(&mut self, spi: &mut hal_Spi, settings: Settings) {
        self.settings = settings;
        self.write_settings(spi)
    }

    fn trig_forced_measure(&mut self, spi: &mut hal_Spi) {
        let data = [CTRL_MEAS & 0x7f, self.settings.ctrl_meas];
        self.ss.set_low();
        spi.write(&data).unwrap();
        self.ss.set_high();
    }
}

