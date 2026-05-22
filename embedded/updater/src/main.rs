use anyhow::Result;
use common::{SpiFlash, flash::region};
use esp_idf_hal::{gpio::PinDriver, peripherals::Peripherals};
use esp_idf_svc::ota::EspOta;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;
    let mut led = PinDriver::output(pins.gpio20)?;
    led.set_high()?;

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

    let mut address = region::UPDATE.start;
    let mut buffer = [0_u8; 512];
    while (address - region::UPDATE.start) < region::UPDATE.len {
        flash.read(address, &mut buffer)?;
        address += buffer.len() as u32;
        update.write(&buffer)?;
    }

    update.complete()?;
    led.set_low()?;
    esp_idf_hal::reset::restart();
}
