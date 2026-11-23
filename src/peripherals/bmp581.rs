use bmp581::{
    I2cAddr,
    bmp581::Bmp581,
    interface::I2cInterface,
    types::{Odr, Osr, OsrConfig, PowerMode},
};
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

#[derive(Debug, Clone, Copy)]
pub struct Bmp581Reading {
    pub temperature_c: f32,
    pub pressure_pa: f32,
}

pub fn init_bmp581<I2C, Delay>(
    i2c: I2C,
    address: I2cAddr,
    delay: &mut Delay,
) -> Result<Bmp581<I2cInterface<I2C>>, bmp581::types::Error<I2C::Error>>
where
    I2C: I2c,
    Delay: DelayNs,
{
    let mut sensor = Bmp581::new_i2c(i2c, address);
    sensor.init(delay)?;
    sensor.set_pressure_enabled(true)?;
    sensor.set_osr_config(OsrConfig {
        pressure_enable: true,
        osr_pressure: Osr::Osr16,
        osr_temperature: Osr::Osr2,
    })?;
    sensor.set_odr(Odr::Hz25_005)?;
    sensor.set_power_mode(PowerMode::Normal)?;
    Ok(sensor)
}

pub fn collect_bmp581<I2C>(
    sensor: &mut Bmp581<I2cInterface<I2C>>,
) -> Result<Bmp581Reading, bmp581::types::Error<I2C::Error>>
where
    I2C: I2c,
{
    Ok(Bmp581Reading {
        temperature_c: sensor.read_temperature()?,
        pressure_pa: sensor.read_pressure()?,
    })
}
