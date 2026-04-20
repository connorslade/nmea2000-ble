use std::{mem, sync::Arc, thread};

use anyhow::Result;
use esp_idf_hal::{
    can::{
        CAN, CanConfig, CanDriver, Frame,
        config::{Filter, Timing},
    },
    delay,
    gpio::{InputPin, OutputPin},
    sys::twai_message_t,
};
use log::{error, info};
use nmea2000::{Header, packets::Packet};

use crate::app::App;

pub fn init(
    app: Arc<App>,
    can: CAN<'static>,
    rx: impl InputPin + 'static,
    tx: impl OutputPin + 'static,
) -> Result<()> {
    let config = CanConfig::new()
        .timing(Timing::B500K) // act gonna kms over this
        .filter(Filter::extended_allow_all());
    let mut can = CanDriver::new(can, tx, rx, &config)?;
    can.start()?;

    thread::spawn(move || {
        loop {
            match can.receive(delay::BLOCK) {
                Ok(frame) => {
                    let frame = unsafe { mem::transmute::<Frame, twai_message_t>(frame) };

                    let header = Header::deserialize(frame.identifier);
                    let packet = Packet::deserialize(header.pgn, frame.data);
                    let Some(packet) = packet else { continue };
                    on_packet(&app, packet);
                }
                Err(err) => error!("CAN receive error: {err}"),
            }
        }
    });

    info!("Initialized CAN");
    Ok(())
}

fn on_packet(app: &App, packet: Packet) {
    match packet {
        Packet::IsoRequest(packet) => {
            info!("Request for PGN {}", packet.pgn);
        }
        Packet::PositionRapidUpdate(packet) => {
            app.position_update(packet.latitude, packet.longitude);
        }
        Packet::CogSogRapidUpdate(packet) => {
            app.speed_update(packet.sog);
        }
        _ => {}
    }
}
