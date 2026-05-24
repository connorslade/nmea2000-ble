use std::{
    io::{BufReader, Read, Write, stdin},
    net::{SocketAddr, TcpStream},
    sync::mpsc::sync_channel,
    thread,
};

use anyhow::{Result, bail};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use nmea2000::{
    Header, Nmea2000,
    packets::{
        Packet,
        proprietary::{SimnetAp, SimnetAp2},
    },
};

fn main() -> Result<()> {
    println!("Searching for windlink");
    let service = find_service()?;
    println!("Found!");

    let mut socket = TcpStream::connect(service)?;
    let mut reader = BufReader::new(socket.try_clone()?);

    let mut nmea2000 = Nmea2000::new().with_preferred_address(0x90);

    let (tx, rx) = sync_channel(10);
    thread::spawn(move || {
        let mut stdin = stdin();
        loop {
            let out = &mut [0];
            if let Ok(1) = stdin.read(out)
                && out[0] == b'\n'
            {
                tx.send(Packet::SimnetAp2(SimnetAp2 {
                    a: 0x00,
                    b: 0x00,
                    c: 0xFE,
                    d: 0xF8,
                    e: 0x00,
                }))
                .unwrap();

                tx.send(Packet::SimnetAp(SimnetAp {
                    address: 6,
                    proprietary: 255,
                    command: 10,
                    event: 6,
                }))
                .unwrap();
            }
        }
    });

    loop {
        let ident = u32::from_be_bytes(read_bytes::<4>(&mut reader)?);
        let header = Header::deserialize(ident);

        let mut data = [0_u8; 8];
        let length = read_bytes::<1>(&mut reader)?[0] as usize;
        reader.read_exact(&mut data[..length])?;

        if let Some(packet) = nmea2000.on_packet(ident, data) {
            match packet {
                Packet::SimnetAp(packet) => println!("{header:?} {packet:?}"),
                Packet::SimnetAp2(packet) => println!("{header:?} {packet:?}"),
                // Packet::SimnetApStatus(packet) => println!("{header:?} {packet:?}"),
                _ => {}
            }
        }

        while let Ok(packet) = rx.try_recv() {
            nmea2000.enqueue(packet, 0xFF);
        }

        for packet in nmea2000.dequeue() {
            println!("Sending {packet:?}");
            socket.write_all(&packet.id.to_be_bytes())?;
            socket.write_all(&[8])?;
            socket.write_all(&packet.data)?;
            socket.flush()?;
        }
    }
}

fn find_service() -> Result<SocketAddr> {
    let mdns = ServiceDaemon::new()?;
    let receiver = mdns.browse("_windlink._tcp.local.")?;

    while let Ok(event) = receiver.recv() {
        match event {
            ServiceEvent::ServiceResolved(service) => {
                let ip = service.addresses.iter().next().unwrap().to_ip_addr();
                return Ok(SocketAddr::new(ip, service.port));
            }
            _ => {}
        }
    }

    bail!("Couldn't find service")
}

fn read_bytes<const N: usize>(mut reader: impl Read) -> Result<[u8; N]> {
    let mut out = [0; N];
    reader.read_exact(&mut out)?;
    Ok(out)
}
