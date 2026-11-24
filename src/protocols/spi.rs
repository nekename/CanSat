pub struct AsyncSpiAdapter<T>(pub T);

impl<T> embedded_hal_async::spi::ErrorType for AsyncSpiAdapter<T>
where
    T: embedded_hal::spi::ErrorType,
{
    type Error = T::Error;
}

impl<T> embedded_hal_async::spi::SpiDevice<u8> for AsyncSpiAdapter<T>
where
    T: embedded_hal::spi::SpiDevice<u8>,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        embedded_hal::spi::SpiDevice::read(&mut self.0, buf)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        embedded_hal::spi::SpiDevice::write(&mut self.0, buf)
    }

    async fn transaction(
        &mut self,
        operations: &mut [embedded_hal::spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        embedded_hal::spi::SpiDevice::transaction(&mut self.0, operations)
    }
}
