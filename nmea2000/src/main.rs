//! Reference: https://canboat.github.io/canboat/canboat.html

use anyhow::{Context, Result};
use socketcan::{
    CanDataFrame, CanInterface, CanSocket, EmbeddedFrame, ExtendedId, Id, Socket,
    available_interfaces,
};

use crate::packets::{Packet, handshake::AddressClaim};

pub mod packets;
mod util;

fn main() -> Result<()> {
    let available = available_interfaces()?;
    let ifname = available.first().context("No CAN interfaces.")?;
    let iface = CanInterface::open(ifname)?;
    let socket = CanSocket::open(ifname)?;

    iface.bring_down()?;
    iface.set_bitrate(250_000, None)?;
    iface.bring_up()?;

    loop {
        match socket.read_frame() {
            Ok(frame) => {
                let Id::Extended(id) = frame.id() else {
                    continue;
                };

                let header = Header::deserialize(id.as_raw());
                let data = u64::from_le_bytes(*frame.data().as_array().unwrap());
                let packet = Packet::deserialize(header.pgn, data);
                println!("{header:?}: {data:X}");

                let Some(packet) = packet else {
                    continue;
                };

                println!("{packet:?}");
                match packet {
                    Packet::AddressClaim(_) => {
                        let header = Header::new(AddressClaim::PGN, 6, 11);
                        let frame = AddressClaim {
                            unique_number: 1824691,
                            manufacturer_code: 2000,
                            device_instance_lower: 0,
                            device_instance_upper: 0,
                            device_function: 150,
                            device_class: 80,
                            system_instance: 0,
                            arbitrary_address_capable: false,
                        };
                        socket.write_frame(
                            &CanDataFrame::new(
                                ExtendedId::new(header.serialize()).unwrap(),
                                &frame.serialize().to_le_bytes(),
                            )
                            .unwrap(),
                        )?;
                    }
                    _ => {}
                }
            }
            Err(err) => eprintln!("{err}"),
        }
    }
}

#[derive(Debug)]
struct Header {
    priority: u8,
    source: u8,
    pgn: u32,
}

impl Header {
    pub fn new(pgn: u32, priority: u8, source: u8) -> Self {
        Self {
            priority,
            source,
            pgn,
        }
    }

    pub fn deserialize(id: u32) -> Self {
        Self {
            priority: ((id >> 26) & 0x7) as u8,
            source: (id & 0xFF) as u8,
            pgn: {
                let data_page = (id >> 24) & 1;
                let pdu_format = ((id >> 16) & 0xFF) as u8;
                let pdu_specific = ((id >> 8) & 0xFF) as u8;
                if pdu_format < 0xF0 {
                    (data_page << 16) | ((pdu_format as u32) << 8)
                } else {
                    (data_page << 16) | ((pdu_format as u32) << 8) | (pdu_specific as u32)
                }
            },
        }
    }

    fn serialize(&self) -> u32 {
        let data_page = (self.pgn >> 16) & 1;
        let pdu_format = (self.pgn >> 8) & 0xFF;
        let pdu_specific = if pdu_format < 0xF0 {
            0xFF
        } else {
            self.pgn & 0xFF
        };

        ((self.priority as u32 & 0x7) << 26)
            | (data_page << 24)
            | (pdu_format << 16)
            | (pdu_specific << 8)
            | (self.source as u32 & 0xFF)
    }
}
