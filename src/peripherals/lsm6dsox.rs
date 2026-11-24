use embedded_hal::{delay::DelayNs, i2c::I2c};
use lsm6dsox::accelerometer::RawAccelerometer;
use lsm6dsox::{
    AccelerometerScale, DataRate, GyroscopeScale, Lsm6dsox as Lsm6dsoxDriver, SlaveAddress,
};

const ACCEL_16G_G_PER_LSB: f32 = 0.000488;

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Lsm6dsoxReading {
    pub accel_x_g: f32,
    pub accel_y_g: f32,
    pub accel_z_g: f32,
    pub gyro_x_mdps: i32,
    pub gyro_y_mdps: i32,
    pub gyro_z_mdps: i32,
}

pub fn init_lsm6dsox<I2C, Delay>(
    i2c: I2C,
    delay: Delay,
    address: SlaveAddress,
) -> Result<Lsm6dsoxDriver<I2C, Delay>, lsm6dsox::types::Error<I2C::Error>>
where
    I2C: I2c,
    Delay: DelayNs,
{
    let mut sensor = Lsm6dsoxDriver::new(i2c, address, delay);
    sensor.setup()?;
    match sensor.check_id() {
        Ok(_) => {}
        Err(_) => return Err(lsm6dsox::types::Error::InvalidData),
    }
    sensor.set_accel_sample_rate(DataRate::Freq104Hz)?;
    sensor.set_accel_scale(AccelerometerScale::Accel16g)?;
    sensor.set_gyro_sample_rate(DataRate::Freq104Hz)?;
    sensor.set_gyro_scale(GyroscopeScale::Dps2000)?;
    Ok(sensor)
}

pub fn collect_lsm6dsox<I2C, Delay>(
    sensor: &mut Lsm6dsoxDriver<I2C, Delay>,
) -> Result<Option<Lsm6dsoxReading>, lsm6dsox::types::Error<I2C::Error>>
where
    I2C: I2c,
    Delay: DelayNs,
{
    let accel = match sensor.accel_raw() {
        Ok(accel) => accel,
        Err(error) => match error.into_cause() {
            lsm6dsox::types::Error::NoDataReady => return Ok(None),
            error => return Err(error),
        },
    };
    let gyro = match sensor.angular_rate_raw() {
        Ok(gyro) => gyro,
        Err(lsm6dsox::types::Error::NoDataReady) => return Ok(None),
        Err(error) => return Err(error),
    };

    Ok(Some(Lsm6dsoxReading {
        accel_x_g: f32::from(accel.x) * ACCEL_16G_G_PER_LSB,
        accel_y_g: f32::from(accel.y) * ACCEL_16G_G_PER_LSB,
        accel_z_g: f32::from(accel.z) * ACCEL_16G_G_PER_LSB,
        gyro_x_mdps: i32::from(gyro.x) * 70,
        gyro_y_mdps: i32::from(gyro.y) * 70,
        gyro_z_mdps: i32::from(gyro.z) * 70,
    }))
}
