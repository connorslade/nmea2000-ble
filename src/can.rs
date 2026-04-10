use std::thread;

use anyhow::Result;
use esp_idf_hal::{
    can::{CAN, CanConfig, CanDriver, Flags, Frame, config::Timing},
    delay,
    gpio::{InputPin, OutputPin},
};
use log::{error, info};

pub fn init(
    can: CAN<'static>,
    rx: impl InputPin + 'static,
    tx: impl OutputPin + 'static,
) -> Result<()> {
    let config = CanConfig::new().timing(Timing::B250K);
    let mut can = CanDriver::new(can, tx, rx, &config)?;
    can.start()?;

    info!("Sending CAN frame");
    can.transmit(
        &Frame::new(100, Flags::None.into(), &[1, 2, 3, 4]).unwrap(),
        delay::BLOCK,
    )?;
    info!("Sent CAN frame");

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
