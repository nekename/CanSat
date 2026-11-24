use lis3mdl::{FullScale, Lis3mdl as Lis3mdlDriver, MeasurementMode, OperatingMode};

const I2C_ADDRESS: u8 = 0x1c;

pub struct Lis3mdlAddressAdapter<I2C>(I2C);

impl<I2C> embedded_hal_0_2::blocking::i2c::Write for Lis3mdlAddressAdapter<I2C>
where
    I2C: embedded_hal_0_2::blocking::i2c::Write,
{
    type Error = I2C::Error;

    fn write(&mut self, _address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.0.write(I2C_ADDRESS, bytes)
    }
}

impl<I2C> embedded_hal_0_2::blocking::i2c::WriteRead for Lis3mdlAddressAdapter<I2C>
where
    I2C: embedded_hal_0_2::blocking::i2c::WriteRead,
{
    type Error = I2C::Error;

    fn write_read(
        &mut self,
        _address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.0.write_read(I2C_ADDRESS, bytes, buffer)
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Lis3mdlReading {
    pub magnetic_x_mgauss: i32,
    pub magnetic_y_mgauss: i32,
    pub magnetic_z_mgauss: i32,
}

pub fn init_lis3mdl<I2C, E>(
    i2c: I2C,
) -> Result<Lis3mdlDriver<Lis3mdlAddressAdapter<I2C>>, lis3mdl::Error>
where
    I2C: embedded_hal_0_2::blocking::i2c::Write<Error = E>
        + embedded_hal_0_2::blocking::i2c::WriteRead<Error = E>,
{
    let mut sensor = Lis3mdlDriver::new(Lis3mdlAddressAdapter(i2c))?;
    sensor.set_full_scale(FullScale::Fs16g)?;
    sensor.set_operating_mode(OperatingMode::UltraHighPerformance)?;
    sensor.set_measurement_mode(MeasurementMode::Continuous)?;
    sensor.set_data_rate(lis3mdl::DataRate::ODR_40Hz)?;
    Ok(sensor)
}

pub fn collect_lis3mdl<I2C, E>(
    sensor: &mut Lis3mdlDriver<I2C>,
) -> Result<Lis3mdlReading, lis3mdl::Error>
where
    I2C: embedded_hal_0_2::blocking::i2c::Write<Error = E>
        + embedded_hal_0_2::blocking::i2c::WriteRead<Error = E>,
{
    let magnetic = sensor.get_mag_axes_mgauss()?;
    Ok(Lis3mdlReading {
        magnetic_x_mgauss: magnetic.x,
        magnetic_y_mgauss: magnetic.y,
        magnetic_z_mgauss: magnetic.z,
    })
}
