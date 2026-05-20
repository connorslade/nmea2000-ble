use anyhow::Result;
use common::SpiFlash;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::ota::EspOta;

const UPDATE_SIZE: u32 = 0x200000;
const FLASH_SIZE: u32 = 0x7A1200;

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

    let mut address = FLASH_SIZE - UPDATE_SIZE;
    let mut buffer = [0; 512];
    while address < FLASH_SIZE {
        flash.read(address, &mut buffer)?;
        address += buffer.len() as u32;
        update.write(&buffer)?;
    }

    update.complete()?;
    esp_idf_hal::reset::restart();
}
