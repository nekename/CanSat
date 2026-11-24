use crate::DataFrame;

use embassy_time::Delay;
use lora_phy::{
    LoRa,
    mod_params::{Bandwidth, CodingRate, RadioError, SpreadingFactor},
    mod_traits::InterfaceVariant,
    sx126x::{Config, Sx126x, Sx1262, TcxoCtrlVoltage},
};

pub const FREQUENCY_HZ: u32 = 868_000_000;
pub const SPREADING_FACTOR: SpreadingFactor = SpreadingFactor::_7;
pub const BANDWIDTH: Bandwidth = Bandwidth::_125KHz;
pub const CODING_RATE: CodingRate = CodingRate::_4_5;
pub const PREAMBLE_LENGTH: u16 = 8;
pub const TX_OUTPUT_POWER_DBM: i32 = 14;

pub fn encode_packet(frame: &DataFrame) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec(frame)
}

pub async fn init_lora<SPI, IV>(
    spi: SPI,
    interface: IV,
) -> Result<LoRa<Sx126x<SPI, IV, Sx1262>, Delay>, RadioError>
where
    SPI: embedded_hal_async::spi::SpiDevice<u8>,
    IV: InterfaceVariant,
{
    let radio = Sx126x::new(
        spi,
        interface,
        Config {
            chip: Sx1262,
            tcxo_ctrl: Some(TcxoCtrlVoltage::Ctrl1V8),
            use_dcdc: true,
            rx_boost: false,
        },
    );
    LoRa::new(radio, false, Delay).await
}

pub async fn transmit_packet<RK, DLY>(
    lora: &mut LoRa<RK, DLY>,
    packet: &[u8],
) -> Result<(), RadioError>
where
    RK: lora_phy::mod_traits::RadioKind,
    DLY: lora_phy::DelayNs,
{
    let modulation =
        lora.create_modulation_params(SPREADING_FACTOR, BANDWIDTH, CODING_RATE, FREQUENCY_HZ)?;
    let mut packet_params =
        lora.create_tx_packet_params(PREAMBLE_LENGTH, false, true, false, &modulation)?;
    lora.prepare_for_tx(&modulation, &mut packet_params, TX_OUTPUT_POWER_DBM, packet)
        .await?;
    lora.tx().await
}
