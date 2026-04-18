use crate::{packets::Packet, util::bits};

/// PGN 60928 - ISO Address Claim
#[derive(Debug)]
pub struct AddressClaim {
    pub unique_number: u32,
    pub manufacturer_code: u16,
    pub device_instance_lower: u8,
    pub device_instance_upper: u8,
    pub device_function: u8,
    pub device_class: u8,
    pub system_instance: u8,
    pub arbitrary_address_capable: bool,
}

impl Packet for AddressClaim {
    const PGN: u32 = 0xEE00;

    fn deserialize(data: u64) -> Self {
        Self {
            unique_number: (data & bits(21)) as _,
            manufacturer_code: (data >> 21 & bits(11)) as _,
            device_instance_lower: (data >> 32 & bits(3)) as _,
            device_instance_upper: (data >> 35 & bits(5)) as _,
            device_function: (data >> 40 & bits(8)) as _,
            device_class: (data >> 49 & bits(7)) as _,
            system_instance: (data >> 56 & bits(4)) as _,
            arbitrary_address_capable: (data >> 60 & bits(1)) != 0,
        }
    }

    fn serialize(&self) -> u64 {
        (self.unique_number as u64)
            | (self.manufacturer_code as u64) << 21
            | (self.device_instance_lower as u64) << 32
            | (self.device_instance_upper as u64) << 35
            | (self.device_function as u64) << 40
            | (self.device_class as u64) << 49
            | (self.system_instance as u64) << 56
            | (self.arbitrary_address_capable as u64) << 60
    }
}
