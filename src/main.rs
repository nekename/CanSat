mod peripherals;
mod protocols;
mod telemetry;

use peripherals::{
    bmp581::{collect_bmp581, init_bmp581},
    lis3mdl::{collect_lis3mdl, init_lis3mdl},
    lsm6dsox::{collect_lsm6dsox, init_lsm6dsox},
    pa1010d::{collect_pa1010d, init_pa1010d},
};
use protocols::{i2c::SharedI2c, spi::AsyncSpiAdapter};
use telemetry::{
    lora::{encode_packet, init_lora, transmit_packet},
    sd::FlightCsvWriter,
};

use std::thread;
use std::time::Duration;

use esp_idf_hal::{
    delay::Delay,
    gpio::{AnyIOPin, PinDriver, Pull},
    i2c::{I2cConfig, I2cDriver, I2cError},
    peripherals::Peripherals,
    sd::{SdCardConfiguration, SdCardDriver, spi::SdSpiHostDriver},
    spi::{
        Dma, SpiDeviceDriver, SpiDriver,
        config::{Config as SpiConfig, DriverConfig},
    },
    task::block_on,
    units::Hertz,
};
use esp_idf_svc::{fs::fatfs::Fatfs, io::vfs::MountedFatfs};
use lora_phy::iv::GenericSx126xInterfaceVariant;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DataFrame {
    pub bmp: peripherals::bmp581::Bmp581Reading,
    pub lis: peripherals::lis3mdl::Lis3mdlReading,
    pub imu: peripherals::lsm6dsox::Lsm6dsoxReading,
    pub gps: peripherals::pa1010d::Pa1010dReading,
}

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

    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        peripherals.pins.gpio12,
        peripherals.pins.gpio11,
        Some(peripherals.pins.gpio13),
        &DriverConfig::default().dma(Dma::Auto(4096)),
    )?;
    let sd_card_driver = SdCardDriver::new_spi(
        SdSpiHostDriver::new(
            &spi_driver,
            Some(peripherals.pins.gpio7),
            AnyIOPin::none(),
            AnyIOPin::none(),
            AnyIOPin::none(),
            None,
        )?,
        &SdCardConfiguration::new(),
    )?;
    let _mounted_fatfs = MountedFatfs::mount(Fatfs::new_sdcard(0, sd_card_driver)?, "/sdcard", 4)?;
    let mut flight_csv = FlightCsvWriter::open_flight_csv()?;

    let lora_reset = PinDriver::output(peripherals.pins.gpio5)?;
    let lora_busy = PinDriver::input(peripherals.pins.gpio4, Pull::Floating)?;
    let lora_dio1 = PinDriver::input(peripherals.pins.gpio1, Pull::Floating)?;
    let lora_interface =
        GenericSx126xInterfaceVariant::new(lora_reset, lora_dio1, lora_busy, None, None)
            .map_err(|error| anyhow::anyhow!("LoRa interface initialization failed: {error:?}"))?;
    let lora_spi = SpiDeviceDriver::new(
        &spi_driver,
        Some(peripherals.pins.gpio10),
        &SpiConfig::new()
            .baudrate(Hertz(1_000_000))
            .data_mode(esp_idf_hal::spi::config::MODE_0),
    )?;
    let mut lora = block_on(init_lora(AsyncSpiAdapter(lora_spi), lora_interface))
        .map_err(|error| anyhow::anyhow!("LoRa initialization failed: {error:?}"))?;
    let mut frame_number = 0_u64;

    while collect_lsm6dsox(&mut lsm6dsox)
        .map_err(|error| anyhow::anyhow!("LSM6DSOX read failed: {error:?}"))?
        .is_none()
    {
        log::debug!("LSM6DSOX not yet ready; skipping this cycle");
        thread::sleep(Duration::from_millis(10));
    }

    loop {
        let bmp = collect_bmp581(&mut bmp581)
            .map_err(|error| anyhow::anyhow!("BMP581 read failed: {error:?}"))?;
        let lis = collect_lis3mdl(&mut lis3mdl)
            .map_err(|error| anyhow::anyhow!("LIS3MDL read failed: {error:?}"))?;
        let imu = collect_lsm6dsox(&mut lsm6dsox)
            .map_err(|error| anyhow::anyhow!("LSM6DSOX read failed: {error:?}"))?
            .unwrap();
        let gps = collect_pa1010d(&mut pa1010d)
            .map_err(|error| anyhow::anyhow!("PA1010D read failed: {error:?}"))?;

        let data = DataFrame { bmp, lis, imu, gps };
        frame_number = frame_number.saturating_add(1);
        if frame_number.is_multiple_of(10) {
            flight_csv
                .append_frame(&data)
                .map_err(|error| anyhow::anyhow!("SD flight frame append failed: {error:?}"))?;
            let packet = encode_packet(&data)
                .map_err(|error| anyhow::anyhow!("LoRa packet serialization failed: {error:?}"))?;
            block_on(transmit_packet(&mut lora, &packet))
                .map_err(|error| anyhow::anyhow!("LoRa transmission failed: {error:?}"))?;
            println!("{:02x?}", packet);
        }

        thread::sleep(Duration::from_millis(100));
    }
}
