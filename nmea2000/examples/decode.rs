//! Reference: https://canboat.github.io/canboat/canboat.html

use anyhow::{Context, Result};
use socketcan::{CanInterface, CanSocket, Socket, available_interfaces};

use nmea2000::{Header, packets::Packet};

fn main() -> Result<()> {
    let available = available_interfaces()?;
    let ifname = available.first().context("No CAN interfaces.")?;
    let iface = CanInterface::open(ifname)?;
    let socket = CanSocket::open(ifname)?;

    iface.bring_down()?;
    iface.set_bitrate(250_000, None)?;
    iface.bring_up()?;

    // let header = Header::new(AddressClaim::PGN, 6, 11);
    // let frame = AddressClaim {
    //     unique_number: 1824692,
    //     manufacturer_code: 2000,
    //     device_instance_lower: 0,
    //     device_instance_upper: 0,
    //     device_function: 150,
    //     device_class: 80,
    //     system_instance: 0,
    //     arbitrary_address_capable: false,
    // };
    // socket.write_frame(
    //     &CanDataFrame::new(
    //         ExtendedId::new(header.serialize()).unwrap(),
    //         &frame.serialize().to_le_bytes(),
    //     )
    //     .unwrap(),
    // )?;

    loop {
        match socket.read_raw_frame() {
            Ok(frame) => {
                let header = Header::deserialize(frame.can_id);
                let packet = Packet::deserialize(header.pgn, frame.data);
                // println!("{header:?}: {data:X}");

                if let Some(packet) = packet {
                    println!("{packet:?}");
                }
            }
            Err(err) => eprintln!("{err}"),
        }
    }
}
