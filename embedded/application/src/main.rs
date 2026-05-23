#![feature(iter_intersperse)]

use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use clone_macro::clone;
use esp_idf_hal::peripherals::Peripherals;

use crate::{
    app::{App, MemoryLogger},
    ble::characteristics::Characteristic,
    util::ForceLock,
};

use common::SpiFlash;

mod app;
mod ble;
mod can;
mod indicator;
mod util;
mod wifi;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let (wifi_modem, _, ble_modem) = peripherals.modem.split();
    let pins = peripherals.pins;
    let ledc = peripherals.ledc;

    let flash = SpiFlash::new(
        peripherals.spi2,
        pins.gpio14,
        pins.gpio13,
        pins.gpio12,
        pins.gpio15,
        40,
    )?;

    let app = Arc::new(App::new(flash)?);
    let logger = Box::new(MemoryLogger::new(&app));
    log::set_logger(Box::leak(logger))?;
    log::set_max_level(log::LevelFilter::Info);

    thread::spawn(clone!([app], move || {
        loop {
            let timer = Instant::now();
            if let Some(bt) = &*app.bt.force_lock() {
                app.boat().notify(bt, Characteristic::WindScreen);
                app.boat().notify(bt, Characteristic::DataScreen);
            }
            thread::sleep(Duration::from_millis(100) - timer.elapsed());
        }
    }));

    wifi::init(app.clone(), wifi_modem)?;
    ble::init(app.clone(), ble_modem)?;
    can::init(app.clone(), peripherals.can, pins.gpio5, pins.gpio6)?;
    indicator::init(app.clone(), ledc.channel0, ledc.timer0, pins.gpio20)?;

    loop {
        thread::park();
    }
}
