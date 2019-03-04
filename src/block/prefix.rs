use super::cursor::BufferCursor;
use crate::error::{Error, Result};
use bitflags::bitflags;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use crc::crc16;
use num::FromPrimitive;
use std::hash::Hasher;

#[derive(Debug, Copy, Clone, FromPrimitive, Eq, PartialEq)]
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
    struct PrefixFlags: u16 {
        const HAS_ADD_SIZE = 0x8000;
        const IS_DELETED = 0x4000;

        // Available on HeadType == FileHeader
        const HAS_SALT = 0x400;
        const HAS_EXT_TIME = 0x1000;
    }
}

// const BLOCK_HEAD_SIZE: usize = 7;

// #[derive(Debug, Clone, PartialEq, Default)]
// pub struct BlockHead {
//     crc: u16,
//     typ: u8,
//     flags: u16,
//     size: u16,
//     add_size: u32,
// }

// impl BlockHead {
//     pub fn block_type(&self) -> Option<HeadType> {
//         HeadType::from_u8(self.typ)
//     }

//     pub fn block_size(&self) -> u64 {
//         u64::from(self.size) + u64::from(self.add_size)
//     }

//     pub fn read(mut cur: impl io::Read) -> Result<Self> {
//         let mut minblock: [u8; BLOCK_HEAD_SIZE] = unsafe { ::std::mem::uninitialized() };
//         cur.read_exact(&mut minblock)
//             .or(Err(Error::buffer_too_small(BLOCK_HEAD_SIZE)))?;

//         let flags = LittleEndian::read_u16(&minblock[3..5]);
//         let add_size = if flags & PrefixFlags::HAS_ADD_SIZE.bits() > 0 {
//             cur.read_u32::<LittleEndian>()
//                 .or(Err(Error::buffer_too_small(BLOCK_HEAD_SIZE + 4)))?
//         } else {
//             0
//         };

//         Ok(Self {
//             crc: LittleEndian::read_u16(&minblock[0..2]),
//             typ: minblock[2],
//             flags: flags,
//             size: LittleEndian::read_u16(&minblock[5..]),
//             add_size: add_size,
//         })
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BlockPrefix<'a> {
    // FIELD BYTES
    // HEAD_CRC 2
    // HEAD_TYPE 1
    // HEAD_FLAGS 2
    // HEAD_SIZE 2
    // ADD_SIZE 4 (optional)
    main: &'a [u8],
    // add_size: Option<&'a [u8]>,
}

impl<'a> BlockPrefix<'a> {
    pub fn crc(&self) -> u16 {
        LittleEndian::read_u16(&self.main[0..2])
    }

    pub fn crc_digest(&self, seed: u16) -> crc16::Digest {
        // panic!("this method is broken still.");
        let mut digest = crc::crc16::Digest::new(seed);
        digest.write(&self.main[2..]);
        // if let Some(ref x) = self.add_size {
        //     digest.write(x);
        // }
        return digest;
    }

    pub fn raw_block_type(&self) -> u8 {
        self.main[2]
    }

    pub fn block_type(&self) -> Option<HeadType> {
        HeadType::from_u8(self.raw_block_type())
    }

    pub fn flags(&self) -> u16 {
        LittleEndian::read_u16(&self.main[3..5])
    }

    pub fn size(&self) -> u64 {
        // let add_size = self
        //     .add_size
        //     .map(|x| LittleEndian::read_u32(x))
        //     .unwrap_or(0);
        // let size = LittleEndian::read_u16(&self.main[5..7]);
        // u64::from(size) + u64::from(add_size)
        u64::from(LittleEndian::read_u16(&self.main[5..7]))
    }

    pub fn from_buf(buf: &'a [u8]) -> Result<(BlockPrefix<'a>, &'a [u8])> {
        let mut cursor = BufferCursor::new(buf);
        let bp = BlockPrefix::from_cursor(&mut cursor)?;
        let rest = cursor.rest();
        Ok((bp, rest))
        // if buf.len() < 7 {
        //     return Err(Error::buffer_too_small(7));
        // }
        // // let flags = LittleEndian::read_u16(&buf[3..5]);

        // // let has_add_size = flags & PrefixFlags::HAS_ADD_SIZE.bits() > 0;
        // // if has_add_size && buf.len() < 7 + 4 {
        // //     return Err(Error::buffer_too_small(7 + 4));
        // // }
        // // let rest = if has_add_size {
        // //     &buf[(7 + 4)..]
        // // } else {
        // //     &buf[7..]
        // // };
        // // let add_size = if has_add_size {
        // //     Some(&buf[7..(7 + 4)])
        // // } else {
        // //     None
        // // };

        // Ok((
        //     BlockPrefix {
        //         main: &buf[0..7],
        //         // add_size: add_size,
        //     },
        //     &buf[7..],
        // ))
    }

    pub fn from_cursor(cursor: &mut BufferCursor<'a>) -> Result<BlockPrefix<'a>> {
        Ok(BlockPrefix {
            main: cursor.read(7)?,
        })
    }

    pub fn as_owned(&self) -> Result<OwnedBlockPrefix> {
        Ok(OwnedBlockPrefix {
            expected_crc: self.crc(),
            block_type: self.block_type().ok_or_else(|| {
                Error::bad_block(format!("Unknown block type: {}", self.raw_block_type()))
            })?,
            flags: self.flags(),
            size: self.size(),
        })
    }
}

#[derive(Clone)]
pub struct OwnedBlockPrefix {
    pub expected_crc: u16,
    pub block_type: HeadType,
    pub flags: u16,
    pub size: u64,
}

impl ::std::fmt::Debug for OwnedBlockPrefix {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(
            f,
            "OwnedBlockPrefix<expected_crc: {:?}, block_type: {:?}, flags: {:?}, size: {:?}>",
            self.expected_crc, self.block_type, self.flags, self.size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn magic_block_prefix() -> Vec<u8> {
        vec![0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00]
    }

    #[test]
    fn test_block_prefix_read_errors_with_not_enough_data() {
        let res = BlockPrefix::from_buf(&[0]);
        assert!(res.is_err());
    }

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
