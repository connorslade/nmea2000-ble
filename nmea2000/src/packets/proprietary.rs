use crate::util::bits;

#[derive(Debug)]
pub struct SimnetAp {
    pub address: u8,
    pub proprietary: u8,
    pub command: u8,
    pub event: u8,
}

impl SimnetAp {
    pub const PGN: u32 = 0x1FF22;

    pub fn deserialize(data: &[u8]) -> Self {
        let manufacturer = u16::from_le_bytes([data[0], data[1] & bits(3)]);
        let industry = (data[1] >> 5) & bits(3);
        assert_eq!(manufacturer, 1857);
        assert_eq!(industry, 4);

        Self {
            address: data[2],
            proprietary: data[4],
            command: data[5],
            event: data[6],
        }
    }

    pub fn serialize(&self) -> [u8; 12] {
        let mut out = [0; 12];
        out[..2].copy_from_slice(&u16::to_le_bytes(1857));
        out[1] |= 4 << 5;
        out[2] = self.address;
        out[3] = 0xFF;
        out[4] = self.proprietary;
        out[5] = self.command;
        out[6] = self.event;
        out[7..12].fill(0xFF);
        out
    }
}

// PGN 65340 - Simnet: AP Unknown 2
#[derive(Debug)]
pub struct SimnetAp2 {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
}

impl SimnetAp2 {
    pub const PGN: u32 = 0xFF3C;

    pub fn deserialize(data: u64) -> Self {
        Self {
            a: (data >> 16 & bits(8)) as _,
            b: (data >> 24 & bits(8)) as _,
            c: (data >> 32 & bits(8)) as _,
            d: (data >> 40 & bits(8)) as _,
            e: (data >> 48 & bits(8)) as _,
        }
    }

    pub fn serialize(&self) -> u64 {
        (1857 | 0b11 << 11 | 0b100 << 13)
            | (self.a as u64) << 16
            | (self.b as u64) << 24
            | (self.c as u64) << 32
            | (self.d as u64) << 40
            | (self.e as u64) << 48
            | (0x80) << 56
    }
}

// PGN 65305 - Simnet: Device Status
#[derive(Debug)]
pub struct SimnetApStatus {
    pub model: u8,
    pub report: u8,
    pub status: u8,
}

impl SimnetApStatus {
    pub const PGN: u32 = 0xFF19;

    pub fn deserialize(data: u64) -> Self {
        Self {
            model: (data >> 16 & bits(8)) as _,
            report: (data >> 24 & bits(8)) as _,
            status: (data >> 32 & bits(8)) as _,
        }
    }

    pub fn serialize(&self) -> u64 {
        (1857 | 0b11 << 11 | 0b100 << 13)
            | (self.model as u64) << 16
            | (self.report as u64) << 24
            | (self.status as u64) << 32
    }
}
