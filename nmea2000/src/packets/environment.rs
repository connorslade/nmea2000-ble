use crate::util::bits;

/// PGN 128267 - Water Depth
#[derive(Debug, Clone)]
pub struct WaterDepth {
    pub sid: u8,
    pub depth: u32,
    pub offset: i16,
    pub range: u8,
}

impl WaterDepth {
    pub const PGN: u32 = 0x1F50B;

    pub fn deserialize(data: u64) -> Self {
        Self {
            sid: (data & bits(8)) as _,
            depth: (data >> 8 & bits(32)) as _,
            offset: (data >> 40 & bits(16)) as _,
            range: (data >> 56 & bits(8)) as _,
        }
    }

    pub fn serialize(&self) -> u64 {
        (self.sid as u64)
            | (self.depth as u64) << 8
            | (self.offset as u64) << 40
            | (self.range as u64) << 56
    }
}

// PGN 130312 - Temperature
#[derive(Debug, Clone)]
pub struct Temperature {
    pub sid: u8,
    pub instance: u8,
    pub source: u8,
    pub temperature: u16,
    pub set_temperature: u16,
}

impl Temperature {
    pub const PGN: u32 = 0x1FD08;

    pub fn deserialize(data: u64) -> Self {
        Self {
            sid: (data & bits(8)) as _,
            instance: (data >> 8 & bits(8)) as _,
            source: (data >> 16 & bits(8)) as _,
            temperature: (data >> 24 & bits(16)) as _,
            set_temperature: (data >> 40 & bits(16)) as _,
        }
    }

    pub fn serialize(&self) -> u64 {
        (self.sid as u64)
            | (self.instance as u64) << 8
            | (self.source as u64) << 16
            | (self.temperature as u64) << 24
            | (self.set_temperature as u64) << 40
    }
}
