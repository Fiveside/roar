use super::BlockPrefix;
use crate::error::{Error, Result};
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use num::FromPrimitive;

#[derive(Debug, Copy, Clone, FromPrimitive, Eq, PartialEq)]
enum OperatingSystem {
    Dos = 0x0,
    OS2 = 0x1,
    Windows = 0x2,
    Unix = 0x3,
    MacOS = 0x4,
    BeOS = 0x5,
}

impl OperatingSystem {
    fn from_u8(that: u8) -> Option<OperatingSystem> {
        FromPrimitive::from_u8(that)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FilePrefix<'a> {
    block_prefix: BlockPrefix<'a>,

    // PACK_SIZE       4                Compressed file size
    // UNP_SIZE        4                Uncompressed file size
    // HOST_OS         1                Operating system used for archiving (See the 'Operating System Indicators' table for the flags used)
    // FILE_CRC        4                File CRC
    // FTIME           4                Date and time in standard MS DOS format
    // UNP_VER         1                RAR version needed to extract file (Version number is encoded as 10 * Major version + minor version.)
    // METHOD          1                Packing method (Please see 'Packing Method' table for all possibilities
    // NAME_SIZE       2                File name size
    // ATTR            4                File attributes
    buf: &'a [u8],
}

impl<'a> FilePrefix<'a> {
    pub fn from_buf(buf: &'a [u8]) -> Result<(FilePrefix<'a>, &'a [u8])> {
        let (prefix, prefix_rest) = BlockPrefix::from_buf(buf)?;
        if prefix_rest.len() < 25 {
            return Err(Error::buffer_too_small(buf.len() - prefix_rest.len() + 25));
        }
        Ok((
            FilePrefix {
                block_prefix: prefix,
                buf: &prefix_rest[0..25],
            },
            &prefix_rest[25..],
        ))
    }

    fn low_compress_size(&self) -> u32 {
        LittleEndian::read_u32(&self.buf[0..4])
    }

    fn low_uncompress_size(&self) -> u32 {
        LittleEndian::read_u32(&self.buf[4..8])
    }

    fn creation_os(&self) -> Option<OperatingSystem> {
        OperatingSystem::from_u8(self.buf[8])
    }
}

// #[derive(Debug, Copy, Clone)]
// pub struct FileHeader<'a> {
//     prefix: BlockPrefix<'a>,
//     // PACK_SIZE       4                Compressed file size
//     // UNP_SIZE        4                Uncompressed file size
//     // HOST_OS         1                Operating system used for archiving (See the 'Operating System Indicators' table for the flags used)
//     // FILE_CRC        4                File CRC
//     // FTIME           4                Date and time in standard MS DOS format
//     // UNP_VER         1                RAR version needed to extract file (Version number is encoded as 10 * Major version + minor version.)
//     // METHOD          1                Packing method (Please see 'Packing Method' table for all possibilities
//     // NAME_SIZE       2                File name size
//     // ATTR            4                File attributes
//     // HIGH_PACK_SIZE  4                High 4 bytes of 64-bit value of compressed file size. Optional value, presents only if bit 0x100 in HEAD_FLAGS is set.
//     // HIGH_UNP_SIZE   4                High 4 bytes of 64-bit value of uncompressed file size. Optional value, presents only if bit 0x100 in HEAD_FLAGS is set.
//     // FILE_NAME       NAME_SIZE bytes  File name - string of NAME_SIZE bytes size
//     // SALT            8                present if (HEAD_FLAGS & 0x400) != 0
//     // EXT_TIME        variable size    present if (HEAD_FLAGS & 0x1000) != 0

//     // holds [PACK_SIZE, ATTR]
//     buf: &'a [u8],

//     // holds [HIGH_PACK_SIZE, HIGH_UNP_SIZE]
//     high_size: Option<&'a [u8]>,

//     // holds file_name
//     file_name: &'a [u8],

//     // holds salt
//     salt: Option<&'a [u8]>,

//     // holds EXT_TIME
//     ext_time: Option<&'a [u8]>,
// }

// impl<'a> FileHeader<'a> {
//     pub fn from_buf(buf: &'a [u8]) -> Result<(FileHeader<'a>, &'a [u8])> {
//         if buf.len() < 25 {
//             return Err(Error::buffer_too_small(25));
//         }
//         let
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    fn prefix_buf() -> Vec<u8> {
        vec![
            192, 218, 116, 128, 144, 94, 0, 154, 182, 5, 0, 154, 182, 5, 0, 2, 216, 13, 113, 119,
            203, 138, 158, 65, 29, 48, 57, 0, 32, 0, 0, 0, 73, 110, 104, 101,
        ]
    }

    #[test]
    fn test_gets_low_compress_size() {
        let buf = prefix_buf();
        let (prefix, _) = FilePrefix::from_buf(&buf).unwrap();
        assert_eq!(prefix.low_compress_size(), 374426);
    }

    #[test]
    fn test_gets_low_uncompress_size() {
        let buf = prefix_buf();
        let (prefix, _) = FilePrefix::from_buf(&buf).unwrap();
        assert_eq!(prefix.low_uncompress_size(), 374426);
    }

    #[test]
    fn test_gets_windows_os() {
        let buf = prefix_buf();
        let (prefix, _) = FilePrefix::from_buf(&buf).unwrap();
        assert_eq!(prefix.creation_os(), Some(OperatingSystem::Windows));
    }
}
