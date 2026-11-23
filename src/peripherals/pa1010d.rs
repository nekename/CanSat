use embedded_hal::i2c::I2c;
use nmea::{Nmea, SentenceType};

pub const PA1010D_I2C_ADDRESS: u8 = 0x10;

#[derive(Debug, Clone, Copy, Default)]
pub struct Pa1010dReading {
    pub latitude_deg: Option<f64>,
    pub longitude_deg: Option<f64>,
    pub altitude_m: Option<f32>,
    pub satellites: Option<u32>,
    pub speed_mps: Option<f32>,
    pub course_deg: Option<f32>,
    pub fix_valid: bool,
}

pub struct Pa1010d<I2C> {
    i2c: I2C,
    parser: Nmea,
}

impl<I2C> Pa1010d<I2C>
where
    I2C: I2c,
{
    fn new(i2c: I2C) -> Result<Self, nmea::Error<'static>> {
        Ok(Self {
            i2c,
            parser: Nmea::create_for_navigation(&[SentenceType::GGA, SentenceType::RMC])?,
        })
    }

    pub fn collect(&mut self) -> Result<Pa1010dReading, I2C::Error> {
        let mut buffer = [0u8; 255];
        self.i2c.read(PA1010D_I2C_ADDRESS, &mut buffer)?;

        let text = String::from_utf8_lossy(&buffer);
        for line in text.split(['\r', '\n']) {
            let line = line.trim_start_matches('\0').trim();
            if line.starts_with('$') {
                let _ = self.parser.parse(line);
            }
        }

        Ok(Pa1010dReading {
            latitude_deg: self.parser.latitude,
            longitude_deg: self.parser.longitude,
            altitude_m: self.parser.altitude,
            satellites: self.parser.num_of_fix_satellites,
            speed_mps: self.parser.speed_over_ground.map(|knots| knots * 0.514444),
            course_deg: self.parser.true_course,
            fix_valid: self
                .parser
                .fix_type()
                .is_some_and(|fix_type| fix_type.is_valid()),
        })
    }
}

pub fn init_pa1010d<I2C>(i2c: I2C) -> Result<Pa1010d<I2C>, nmea::Error<'static>>
where
    I2C: I2c,
{
    Pa1010d::new(i2c)
}

pub fn collect_pa1010d<I2C>(sensor: &mut Pa1010d<I2C>) -> Result<Pa1010dReading, I2C::Error>
where
    I2C: I2c,
{
    sensor.collect()
}
