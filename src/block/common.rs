use crc::{crc32, Hasher32};
use nom::bytes::streaming::tag;
use nom::combinator::consumed;
use nom::number::streaming::{le_u16, le_u32, le_u8};
use nom::sequence::tuple;
use nom::IResult;
use std::fmt::Debug;

pub trait RarFlags {
    fn get_flags(&self) -> u16;

    fn has_add_size(&self) -> bool {
        self.get_flags() & 0x8000 != 0
    }

    fn marked_as_deleted(&self) -> bool {
        self.get_flags() & 0x4000 != 0
    }
}

struct RarCRC(crc32::Digest);

impl RarCRC {
    fn new() -> Self {
        Self(crc32::Digest::new(crc32::IEEE))
    }

    fn sum16(&self) -> u16 {
        // Rar always uses CRC32 internally.  However some areas only have
        // a u16 allocated for the checksum.  Rar just uses the lower bits
        // for this for some reason.
        (self.0.sum32() & 0xFFFF) as u16
    }
}

impl Hasher32 for RarCRC {
    fn reset(&mut self) {
        self.0.reset()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes)
    }

    fn sum32(&self) -> u32 {
        self.0.sum32()
    }
}

impl Debug for RarCRC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RarCRC")
            .field("sum16", &self.sum16())
            .field("sum32", &self.sum32())
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const RAR3_MAGIC: [u8; 7] = [0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x00];

    #[test]
    fn rar_crc_generates_correct_crc16() {
        let mut hasher = RarCRC::new();
        hasher.write(&[0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x00]);
        assert_eq!(hasher.sum16(), 12426)
    }
}
