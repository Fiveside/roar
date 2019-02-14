use bitflags::bitflags;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use num::FromPrimitive;
use std::io;

#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum HeadType {
    MarkerBlock = 0x72,
    ArchiveHeader = 0x73,
    FileHeader = 0x74,
    OldCommentHeader = 0x75,
    OldAuthenticityInformation = 0x76,
    OldSubBlock = 0x77,
    OldRecoveryRecord = 0x78,
    OldAuthenticityInformation2 = 0x79,
    SubBlock = 0x7a,
    Terminator = 0x7b,
}

impl HeadType {
    pub fn from_u8(that: u8) -> Option<HeadType> {
        FromPrimitive::from_u8(that)
    }
}

bitflags! {
    struct BlockFlags: u16 {
        const HAS_ADD_SIZE = 0x8000;
        const IS_DELETED = 0x4000;

        // Available on HeadType == FileHeader
        const HAS_SALT = 0x400;
        const HAS_EXT_TIME = 0x1000;
    }
}

const BLOCK_HEAD_SIZE: usize = 7;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlockHead {
    crc: u16,
    typ: u8,
    flags: u16,
    size: u16,
    add_size: u32,
}

impl BlockHead {
    pub fn block_type(&self) -> Option<HeadType> {
        HeadType::from_u8(self.typ)
    }

    pub fn block_size(&self) -> u64 {
        u64::from(self.size) + u64::from(self.add_size)
    }

    pub fn read(mut cur: impl io::Read) -> Result<Self, io::Error> {
        let mut minblock: [u8; BLOCK_HEAD_SIZE] = unsafe { ::std::mem::uninitialized() };
        cur.read_exact(&mut minblock)?;

        let flags = LittleEndian::read_u16(&minblock[3..5]);
        let add_size = if flags & BlockFlags::HAS_ADD_SIZE.bits() > 0 {
            cur.read_u32::<LittleEndian>()?
        } else {
            0
        };

        Ok(Self {
            crc: LittleEndian::read_u16(&minblock[0..2]),
            typ: minblock[2],
            flags: flags,
            size: LittleEndian::read_u16(&minblock[5..]),
            add_size: add_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn magic_blockhead() -> Vec<u8> {
        vec![0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00]
    }

    #[test]
    fn test_read_block_head_reads_block_head() {
        let block_result = BlockHead::read(&mut Cursor::new(magic_blockhead()));
        assert!(block_result.is_ok());

        let block = block_result.unwrap();
        assert_eq!(block.crc, 0x6152);
        assert_eq!(block.typ, 0x72);
        assert_eq!(block.flags, 0x1a21);
        assert_eq!(block.size, 0x0007);
    }

    #[test]
    fn test_read_block_head_errors_when_not_enough_data() {
        let block_result = BlockHead::read(&mut Cursor::new(vec![0x00]));
        assert!(block_result.is_err());
    }

    #[test]
    fn test_block_head_without_add_size_reports_size() {
        let mut bh: BlockHead = Default::default();
        bh.size = 3;
        assert_eq!(bh.block_size(), 3);
    }

    #[test]
    fn test_block_head_with_add_size_reports_size() {
        let mut bh: BlockHead = Default::default();
        bh.size = 4;
        bh.add_size = 5;
        assert_eq!(bh.block_size(), 9);
    }
}
