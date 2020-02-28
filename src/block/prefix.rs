use crate::error::{Result, RoarError};
use crate::io::{CRC16Reader, FileReader};
use bitflags::bitflags;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use crc::crc16;
use crc::crc16::Hasher16;
use futures::prelude::*;
use futures::{AsyncRead, AsyncReadExt};
use std::hash::Hasher;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HeadType {
    MarkerBlock,
    ArchiveHeader,
    FileHeader,
    OldCommentHeader,
    OldAuthenticityInformation,
    OldSubBlock,
    OldRecoveryRecord,
    OldAuthenticityInformation2,
    SubBlock,
    Terminator,
    Unknown(u8),
}

impl HeadType {
    pub fn from_u8(that: u8) -> Self {
        use HeadType::*;
        match that {
            0x72 => MarkerBlock,
            0x73 => ArchiveHeader,
            0x74 => FileHeader,
            0x75 => OldCommentHeader,
            0x76 => OldAuthenticityInformation,
            0x77 => OldSubBlock,
            0x78 => OldRecoveryRecord,
            0x79 => OldAuthenticityInformation2,
            0x7a => SubBlock,
            0x7b => Terminator,
            _ => Unknown(that),
        }
    }
    pub fn as_u8(&self) -> u8 {
        use HeadType::*;
        match self {
            MarkerBlock => 0x72,
            ArchiveHeader => 0x73,
            FileHeader => 0x74,
            OldCommentHeader => 0x75,
            OldAuthenticityInformation => 0x76,
            OldSubBlock => 0x77,
            OldRecoveryRecord => 0x78,
            OldAuthenticityInformation2 => 0x79,
            SubBlock => 0x7a,
            Terminator => 0x7b,
            Unknown(ref x) => *x,
        }
    }
}

bitflags! {
    struct PrefixFlags: u16 {
        const HAS_ADD_SIZE = 0x8000;
        const IS_DELETED = 0x4000;

        // Available on HeadType == FileHeader
        const HAS_SALT = 0x400;
        const HAS_EXT_TIME = 0x1000;
    }
}

pub struct BlockHeaderCommon {
    pub expected_header_crc: u16,
    pub header_type: HeadType,
    flags: PrefixFlags,
    pub flags_raw: u16,
    header_size: u16,
    additional_size: u32,
}

impl ::std::fmt::Debug for BlockHeaderCommon {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(
            f,
            "BlockHeaderCommon{{ expected_crc: {:?}, header_type: {:?}, header_flags: {:?}, reported_block_size: {:?} }}",
            self.expected_header_crc,
            self.header_type,
            self.flags,
            self.block_size(),
        )
    }
}

impl BlockHeaderCommon {
    pub async fn parse<T: FileReader>(f: &mut CRC16Reader<'_, T>) -> Result<BlockHeaderCommon> {
        let header_crc = f.read_u16().await?;

        let header_type_raw = f.read_u8().await?;
        let header_type = HeadType::from_u8(header_type_raw);

        let header_flags_raw = f.read_u16().await?;
        let header_flags = PrefixFlags::from_bits_truncate(header_flags_raw);
        let header_size = f.read_u16().await?;

        let additional_size = if header_flags.contains(PrefixFlags::HAS_ADD_SIZE) {
            f.read_u32().await?
        } else {
            0
        };

        Ok(BlockHeaderCommon {
            expected_header_crc: header_crc,
            header_type,
            flags: header_flags,
            flags_raw: header_flags_raw,
            header_size,
            additional_size,
        })
    }

    pub fn block_size(&self) -> u32 {
        self.additional_size
            .checked_add(self.header_size as u32)
            .expect("Overflow calculating block size")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn magic_block_prefix() -> Vec<u8> {
        vec![0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00]
    }

    //    #[test]
    //    fn test_block_prefix_read_errors_with_not_enough_data() {
    //        BlockHeaderCommon::read_from_file(Cursor::new(magic_block_prefix()))
    //        let res = BlockPrefix::from_buf(&[0]);
    //        assert!(res.is_err());
    //    }

    // #[test]
    // fn test_block_prefix_read_errors_with_not_enough_data_from_add_data() {
    //     let mut buf = magic_block_prefix();
    //     buf[4] = 0x80;
    //     let res = BlockPrefix::from_buf(&buf);
    //     assert!(res.is_err());
    // }

    #[test]
    fn test_block_prefix_read_reads_magic() {
        let magic = magic_block_prefix();
        let res = BlockPrefix::from_buf(&magic);
        assert!(res.is_ok());

        let (bh, rest) = res.unwrap();
        assert_eq!(bh.crc(), 0x6152);
        assert_eq!(bh.block_type(), Some(HeadType::MarkerBlock));
        assert_eq!(bh.flags(), 0x1a21);
        assert_eq!(bh.size(), 0x0007);

        assert_eq!(rest.len(), 0);
    }

    // #[test]
    // fn test_read_old_block_head_reads_block_head() {
    //     let block_result = BlockHead::read(&mut Cursor::new(magic_blockhead()));
    //     assert!(block_result.is_ok());

    //     let block = block_result.unwrap();
    //     assert_eq!(block.crc, 0x6152);
    //     assert_eq!(block.typ, 0x72);
    //     assert_eq!(block.flags, 0x1a21);
    //     assert_eq!(block.size, 0x0007);
    // }

    // #[test]
    // fn test_read_old_block_head_errors_when_not_enough_data() {
    //     let block_result = BlockHead::read(&mut Cursor::new(vec![0x00]));
    //     assert!(block_result.is_err());
    // }

    // #[test]
    // fn test_old_block_head_without_add_size_reports_size() {
    //     let mut bh: BlockHead = Default::default();
    //     bh.size = 3;
    //     assert_eq!(bh.block_size(), 3);
    // }

    // #[test]
    // fn test_old_block_head_with_add_size_reports_size() {
    //     let mut bh: BlockHead = Default::default();
    //     bh.size = 4;
    //     bh.add_size = 5;
    //     assert_eq!(bh.block_size(), 9);
    // }
}
