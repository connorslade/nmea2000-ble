use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::log::EspLogger;

mod ble;
mod can;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    ble::init(peripherals.modem)?;
    can::init(peripherals.can, pins.gpio4, pins.gpio5)?;

    loop {
        std::thread::park()
    }
}
