use anyhow::Result;
use common::{SpiFlash, flash::UPDATE_REGION};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::ota::EspOta;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    let flash = SpiFlash::new(
        peripherals.spi2,
        pins.gpio14,
        pins.gpio13,
        pins.gpio12,
        pins.gpio15,
        40,
    )?;

    let mut ota = EspOta::new()?;
    let mut update = ota.initiate_update()?;

    let mut address = UPDATE_REGION.0;
    let mut buffer = [0_u8; 512];

    while (address - UPDATE_REGION.0) < UPDATE_REGION.1 {
        flash.read(address, &mut buffer)?;
        address += buffer.len() as u32;
        update.write(&buffer)?;
    }

    update.complete()?;
    esp_idf_hal::reset::restart();
}
