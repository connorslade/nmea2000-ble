#![feature(mapped_lock_guards)]
#![feature(iter_intersperse)]

use std::{sync::Arc, thread};

use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;
use log::info;

use crate::{
    app::{App, MemoryLogger},
    flash::SpiFlash,
};

mod app;
mod ble;
mod can;
mod flash;
mod indicator;
mod util;
mod wifi;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let (wifi_modem, _, ble_modem) = peripherals.modem.split();
    let pins = peripherals.pins;
    let ledc = peripherals.ledc;

    let app = Arc::new(App::new());
    let logger = Box::new(MemoryLogger::new(&app));
    log::set_logger(Box::leak(logger))?;
    log::set_max_level(log::LevelFilter::Info);

    wifi::init(app.clone(), wifi_modem)?;
    ble::init(app.clone(), ble_modem)?;
    can::init(app.clone(), peripherals.can, pins.gpio6, pins.gpio5)?;
    indicator::init(app.clone(), ledc.channel0, ledc.timer0, pins.gpio20)?;

    let flash = SpiFlash::new(
        peripherals.spi2,
        pins.gpio14,
        pins.gpio13,
        pins.gpio12,
        pins.gpio15,
        40,
    )?;

    let id = flash.read_id()?;
    let size = flash.size()?;
    info!("External flash JEDEC=0x{:#08x}, size={} bytes", id, size);

    flash.erase_region(0x0000, 4096)?;

    let data: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
    flash.write(0x0000, &data)?;

    let mut out = [0u8; 9];
    flash.read(0x0000, &mut out)?;
    info!("Read back: {:?}", out);

    loop {
        thread::park();
    }
}
