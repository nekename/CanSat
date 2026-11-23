mod peripherals;
mod protocols;

use peripherals::{
    bmp581::{collect_bmp581, init_bmp581},
    lis3mdl::{collect_lis3mdl, init_lis3mdl},
    lsm6dsox::{collect_lsm6dsox, init_lsm6dsox},
    pa1010d::{collect_pa1010d, init_pa1010d},
};
use protocols::i2c::SharedI2c;

use std::thread;
use std::time::Duration;

use esp_idf_hal::i2c::I2cError;
use esp_idf_hal::{
    delay::Delay,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    units::Hertz,
};

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let i2c_config = I2cConfig::new().baudrate(Hertz(100_000));
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio8,
        peripherals.pins.gpio9,
        &i2c_config,
    )?;
    let i2c = SharedI2c::new(i2c);
    let mut delay = Delay::new_default();

    let mut bmp581 = init_bmp581(i2c.clone(), bmp581::I2cAddr::Alternative, &mut delay)
        .map_err(|error| anyhow::anyhow!("BMP581 initialization failed: {error:?}"))?;
    let mut lis3mdl = init_lis3mdl::<SharedI2c, I2cError>(i2c.clone())
        .map_err(|error| anyhow::anyhow!("LIS3MDL initialization failed: {error:?}"))?;
    let mut lsm6dsox = init_lsm6dsox(i2c.clone(), delay, lsm6dsox::SlaveAddress::Low)
        .map_err(|error| anyhow::anyhow!("LSM6DSOX initialization failed: {error:?}"))?;
    let mut pa1010d = init_pa1010d(i2c)
        .map_err(|error| anyhow::anyhow!("PA1010D initialization failed: {error:?}"))?;

    loop {
        let bmp = collect_bmp581(&mut bmp581)
            .map_err(|error| anyhow::anyhow!("BMP581 read failed: {error:?}"))?;
        let lis = collect_lis3mdl(&mut lis3mdl)
            .map_err(|error| anyhow::anyhow!("LIS3MDL read failed: {error:?}"))?;
        let imu = collect_lsm6dsox(&mut lsm6dsox)
            .map_err(|error| anyhow::anyhow!("LSM6DSOX read failed: {error:?}"))?;
        let gps = collect_pa1010d(&mut pa1010d)
            .map_err(|error| anyhow::anyhow!("PA1010D read failed: {error:?}"))?;

        log::info!(
            "BMP581: {:.2} C, {:.2} Pa | LIS3MDL: {}, {}, {} mgauss | LSM6DSOX: acc ({}, {}, {}) g, gyro ({}, {}, {}) mdps | GPS: lat {:?}, lon {:?}, alt {:?} m, sats {:?}, speed {:?} m/s, course {:?}, fix {}",
            bmp.temperature_c,
            bmp.pressure_pa,
            lis.magnetic_mgauss.x,
            lis.magnetic_mgauss.y,
            lis.magnetic_mgauss.z,
            imu.accel_x_g,
            imu.accel_y_g,
            imu.accel_z_g,
            imu.gyro_x_mdps,
            imu.gyro_y_mdps,
            imu.gyro_z_mdps,
            gps.latitude_deg,
            gps.longitude_deg,
            gps.altitude_m,
            gps.satellites,
            gps.speed_mps,
            gps.course_deg,
            gps.fix_valid,
        );

        thread::sleep(Duration::from_millis(100));
    }
}
