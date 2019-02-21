use super::BlockPrefix;
use crate::error::{Error, Result};
use bitflags::bitflags;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use num::FromPrimitive;
use super::cursor::BufferCursor;

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

#[derive(Debug, Copy, Clone, FromPrimitive, Eq, PartialEq)]
enum PackingMethod {
    Store = 0x30,
    Fastest = 0x31,
    Fast = 0x32,
    Normal = 0x33,
    Good = 0x34,
    Best = 0x35,
}

impl PackingMethod {
    fn from_u8(that: u8) -> Option<PackingMethod> {
        FromPrimitive::from_u8(that)
    }
}

bitflags! {
    struct FileFlags: u16 {
        const ContinuedFromPreviousVolume = 0b0000_0000_0000_0001;
        const ContinuedToNextVolume = 0b0000_0000_0000_0010;
        const Encrypted = 0b0000_0000_0000_0100;
        const CommentPresent = 0b0000_0000_0000_1000;
        const Solid = 0b0000_0000_0001_0000;
        const Dictionary1 = 0b0000_0000_0010_0000;
        const Dictionary2 = 0b0000_0000_0100_0000;
        const Dictionary3 = 0b0000_0000_1000_0000;
        const HighFields = 0b0000_0001_0000_0000;

        // FILE_NAME contains both usual and encoded Unicode name separated by
        // zero. In this case NAME_SIZE field is equal to the length of usual
        // name plus encoded Unicode name plus 1. If this flag is present,
        // but FILE_NAME does not contain zero bytes, it means that file name
        // is encoded using UTF-8.
        const UnicodeFilename = 0b0000_0010_0000_0000;

        const Salted = 0b0000_0100_0000_0000;
        const Versioned = 0b0000_1000_0000_0000;
        const ExtTime = 0b0001_0000_0000_0000;

        // This flag should always be set
        const Always = 0b1000_0000_0000_0000;
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
        let mut cursor = BufferCursor::new(buf);
        let fp = FilePrefix::from_cursor(&mut cursor)?;
        Ok((fp, cursor.rest()))
        // let (prefix, prefix_rest) = BlockPrefix::from_buf(buf)?;
        // if prefix_rest.len() < 25 {
        //     return Err(Error::buffer_too_small(buf.len() - prefix_rest.len() + 25));
        // }
        // Ok((
        //     FilePrefix {
        //         block_prefix: prefix,
        //         buf: &prefix_rest[0..25],
        //     },
        //     &prefix_rest[25..],
        // ))
    }

    pub fn from_cursor(cursor: &mut BufferCursor<'a>) -> Result<FilePrefix<'a>> {
        let prefix = BlockPrefix::from_cursor(cursor)?;
        let buf = cursor.read(25)?;
        Ok(FilePrefix {
            block_prefix: prefix,
            buf: buf,
        })
    }

    fn prefix(&self) -> BlockPrefix {
        self.block_prefix
    }

    fn flags(&self) -> FileFlags {
        FileFlags::from_bits_truncate(self.prefix().flags())
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

    fn file_crc32(&self) -> u32 {
        LittleEndian::read_u32(&self.buf[9..9 + 4])
    }

    fn ftime_raw(&self) -> u32 {
        LittleEndian::read_u32(&self.buf[13..13 + 4])
    }

    // This might need an enum later as well...
    fn unpack_version(&self) -> u8 {
        self.buf[17]
    }

    fn packing_method(&self) -> Option<PackingMethod> {
        PackingMethod::from_u8(self.buf[18])
    }

    fn name_size(&self) -> u16 {
        LittleEndian::read_u16(&self.buf[19..21])
    }

    fn file_attrs(&self) -> u32 {
        LittleEndian::read_u32(&self.buf[21..])
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FileHeader<'a> {
    prefix: FilePrefix<'a>,

    // HIGH_PACK_SIZE 4
    // HIGH_UNPACK_SIZE 4
    // holds [HIGH_PACK_SIZE, HIGH_UNP_SIZE]
    high_size: Option<&'a [u8]>,

    // holds file_name
    file_name: &'a [u8],

    // SALT 8
    // holds salt
    salt: Option<&'a [u8]>,

    // holds EXT_TIME
    ext_time: Option<&'a [u8]>,
}

impl<'a> FileHeader<'a> {
    pub fn from_buf(buf: &'a [u8]) -> Result<(FileHeader<'a>, &'a [u8])> {
        let mut cursor = BufferCursor::new(buf);
        let fh = FileHeader::from_cursor(&mut cursor)?;
        Ok((fh, cursor.rest()))
    }

    pub fn from_cursor(cursor: &mut BufferCursor<'a>) -> Result<FileHeader<'a>> {
        let prefix = FilePrefix::from_cursor(cursor)?;
        let flags = prefix.flags();

        let high_size = if flags.contains(FileFlags::HighFields) {
            Some(cursor.read(8)?)
        } else {
            None
        };

        // let name_size = usize::from(prefix.name_size());
        let name = cursor.read(usize::from(prefix.name_size()))?;

        let salt = if flags.contains(FileFlags::Salted) {
            Some(cursor.read(8)?)
        } else {
            None
        };

        // TODO: ext_time

        Ok(FileHeader {
            prefix: prefix,
            high_size: high_size,
            file_name: name,
            salt: salt,
            ext_time: None,
        })
    }
}

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

    #[test]
    fn test_gets_file_crc32() {
        let buf = prefix_buf();
        let (prefix, _) = FilePrefix::from_buf(&buf).unwrap();
        assert_eq!(prefix.file_crc32(), 2003897816);
    }

    #[test]
    fn test_gets_raw_ftime() {
        let buf = prefix_buf();
        let (prefix, _) = FilePrefix::from_buf(&buf).unwrap();
        assert_eq!(prefix.ftime_raw(), 1100909259);
    }

    #[test]
    fn test_gets_unpack_version() {
        let buf = prefix_buf();
        let (prefix, _) = FilePrefix::from_buf(&buf).unwrap();
        assert_eq!(prefix.unpack_version(), 29);
    }

    #[test]
    fn test_gets_packing_method() {
        let buf = prefix_buf();
        let (prefix, _) = FilePrefix::from_buf(&buf).unwrap();
        assert_eq!(prefix.packing_method(), Some(PackingMethod::Store));
    }

    #[test]
    fn test_gets_name_size() {
        let buf = prefix_buf();
        let (prefix, _) = FilePrefix::from_buf(&buf).unwrap();
        assert_eq!(prefix.name_size(), 57);
    }

    #[test]
    fn test_gets_file_attrs() {
        let buf = prefix_buf();
        let (prefix, _) = FilePrefix::from_buf(&buf).unwrap();
        assert_eq!(prefix.file_attrs(), 32);
    }

    #[test]
    fn test_gets_flags() {
        let buf = prefix_buf();
        let (prefix, _) = FilePrefix::from_buf(&buf).unwrap();
        let expected = FileFlags::Dictionary3 | FileFlags::ExtTime | FileFlags::Always;
        assert_eq!(prefix.flags(), FileFlags::Dictionary3 | expected);
    }
}
