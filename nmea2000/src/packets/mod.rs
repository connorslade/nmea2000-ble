use std::fmt::Debug;

use crate::packets::{
    handshake::{AddressClaim, IsoRequest},
    motion::{CogSogRapidUpdate, PositionRapidUpdate, VesselHeading, WindData},
};

pub mod handshake;
pub mod motion;

#[derive(Debug)]
pub enum Packet {
    IsoRequest(IsoRequest),
    AddressClaim(AddressClaim),
    PositionRapidUpdate(PositionRapidUpdate),
    CogSogRapidUpdate(CogSogRapidUpdate),
    VesselHeading(VesselHeading),
    WindData(WindData),
}

macro_rules! parse_packet {
    ($pgn:expr, $data:expr, [$($ident:ident),*]) => {
        Some(match $pgn {
            $($ident::PGN => Self::$ident($ident::deserialize($data)),)*
            _ => return None,
        })
    };
}

impl Packet {
    pub fn deserialize(pgn: u32, data: u64) -> Option<Self> {
        parse_packet!(
            pgn,
            data,
            [
                IsoRequest,
                AddressClaim,
                PositionRapidUpdate,
                CogSogRapidUpdate,
                VesselHeading,
                WindData
            ]
        )
    }
}

pub trait IPacket: Debug {
    fn serialize(&self) -> u64;
}
