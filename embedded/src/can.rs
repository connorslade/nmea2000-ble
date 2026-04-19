use std::{mem, thread};

use anyhow::Result;
use esp_idf_hal::{
    can::{
        CAN, CanConfig, CanDriver, Flags, Frame,
        config::{Filter, Timing},
    },
    delay,
    gpio::{InputPin, OutputPin},
    sys::twai_message_t,
};
use log::{error, info};
use nmea2000::{Header, packets::Packet};

pub fn init(
    can: CAN<'static>,
    rx: impl InputPin + 'static,
    tx: impl OutputPin + 'static,
) -> Result<()> {
    let config = CanConfig::new()
        .timing(Timing::B500K) // act gonna kms over this
        .filter(Filter::extended_allow_all());
    let mut can = CanDriver::new(can, tx, rx, &config)?;
    can.start()?;

    info!("Transmitting");
    can.transmit(
        &Frame::new(418316043, Flags::Extended.into(), &[0, 1, 2, 3, 4, 5, 6, 7]).unwrap(),
        delay::BLOCK,
    )?;

    info!("Receiving");
    thread::spawn(move || {
        loop {
            match can.receive(delay::BLOCK) {
                Ok(frame) => {
                    let frame = unsafe { mem::transmute::<_, twai_message_t>(frame) };

                    let header = Header::deserialize(frame.identifier);
                    let packet = Packet::deserialize(header.pgn, frame.data);
                    if let Some(packet) = packet {
                        info!("{packet:?}");
                    }
                }
                Err(err) => error!("CAN receive error: {err}"),
            }
        }
    });

    info!("Initialized CAN");
    Ok(())
}
