use std::thread;

use anyhow::Result;
use esp_idf_hal::{
    can::{
        CAN, CanConfig, CanDriver,
        config::{Filter, Timing},
    },
    delay,
    gpio::{InputPin, OutputPin},
};
use log::{error, info};

pub fn init(
    can: CAN<'static>,
    rx: impl InputPin + 'static,
    tx: impl OutputPin + 'static,
) -> Result<()> {
    let config = CanConfig::new()
        .timing(Timing::B250K)
        .filter(Filter::extended_allow_all());
    let mut can = CanDriver::new(can, tx, rx, &config)?;
    can.start()?;

    thread::spawn(move || {
        loop {
            match can.receive(delay::BLOCK) {
                Ok(frame) => {
                    info!("Got CAN frame: {}, {:?}", frame.identifier(), frame.data());
                }
                Err(err) => error!("CAN receive error: {err}"),
            }
        }
    });

    info!("Initialized CAN");
    Ok(())
}
