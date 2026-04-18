use std::fmt::Debug;

pub mod handshake;

pub trait Packet: Debug {
    const PGN: u32;
    fn deserialize(data: u64) -> Self;
    fn serialize(&self) -> u64;
}
