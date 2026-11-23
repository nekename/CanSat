use std::sync::{Arc, Mutex};

use esp_idf_hal::i2c::I2cDriver;
use esp_idf_hal::i2c::I2cError;

#[derive(Clone)]
pub struct SharedI2c(Arc<Mutex<I2cDriver<'static>>>);

impl SharedI2c {
    pub fn new(driver: I2cDriver<'static>) -> Self {
        Self(Arc::new(Mutex::new(driver)))
    }
}

impl embedded_hal::i2c::ErrorType for SharedI2c {
    type Error = I2cError;
}

impl embedded_hal::i2c::I2c for SharedI2c {
    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        embedded_hal::i2c::I2c::read(&mut *self.0.lock().unwrap(), address, read)
    }

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        embedded_hal::i2c::I2c::write(&mut *self.0.lock().unwrap(), address, write)
    }

    fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        embedded_hal::i2c::I2c::write_read(&mut *self.0.lock().unwrap(), address, write, read)
    }

    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        embedded_hal::i2c::I2c::transaction(&mut *self.0.lock().unwrap(), address, operations)
    }
}

impl embedded_hal_0_2::blocking::i2c::Write for SharedI2c {
    type Error = I2cError;

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        embedded_hal_0_2::blocking::i2c::Write::write(&mut *self.0.lock().unwrap(), address, write)
    }
}

impl embedded_hal_0_2::blocking::i2c::WriteRead for SharedI2c {
    type Error = I2cError;

    fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        embedded_hal_0_2::blocking::i2c::WriteRead::write_read(
            &mut *self.0.lock().unwrap(),
            address,
            write,
            read,
        )
    }
}
