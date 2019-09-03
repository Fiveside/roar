use super::cursor::{AsyncCRC16Cursor};
use crate::block::prefix::BlockHeaderCommon;
use crate::error::Result;
use crate::traits::AsyncFile;
use byteorder::{ByteOrder, LittleEndian};
use crc::crc16;
use crc::crc16::Hasher16;
use futures::io::AsyncRead;
use std::hash::Hasher;

struct BlockCRC {
    expected_crc: u16,
    actual_crc: u16,
}

#[derive(Debug)]
pub struct ArchiveHeader {
    // prefix: BlockPrefix<'a>,
    pub prefix: BlockHeaderCommon,
    pub block_crc: u16,

    // Probably HighPosAv
    pub reserved1: u16,

    // Probably PosAv
    pub reserved2: u32,
    // Optional (maybe?) 1 byte EncryptVer (not implemented right now)
}

impl ArchiveHeader {
    // pub fn from_buf(buf: &'a [u8]) -> Result<(ArchiveHeader<'a>, &'a [u8])> {
    //     let mut cursor = BufferCursor::new(buf);
    //     let ah = ArchiveHeader::from_cursor(&mut cursor)?;
    //     Ok((ah, cursor.rest()))
    // }

    // pub fn from_cursor(cursor: &mut BufferCursor<'a>) -> Result<ArchiveHeader<'a>> {
    //     let prefix = BlockPrefix::from_cursor(cursor)?;
    //     Ok(ArchiveHeader {
    //         prefix: prefix,
    //         buf: cursor.read(6)?,
    //     })
    // }

    pub async fn parse<'a>(
        prefix: BlockHeaderCommon,
        f: &'a mut impl AsyncFile,
    ) -> Result<ArchiveHeader> {
        // FIXME: the digest seed is incorrect.
        let mut cursor = AsyncCRC16Cursor::new(f, prefix.digest.sum16());

        let reserved1 = cursor.read_u16().await?;
        let reserved2 = cursor.read_u32().await?;
        Ok(ArchiveHeader {
            prefix: prefix,
            block_crc: cursor.digest.sum16(),
            reserved1: reserved1,
            reserved2: reserved2,
        })
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn archive_header_prefix() -> Vec<u8> {
        vec![207, 144, 115, 0, 0, 13, 0]
    }

    fn archive_header() -> Vec<u8> {
        let mut buf = archive_header_prefix();
        buf.extend(&[1, 0, 2, 0, 0, 0]);
        buf
    }

    // #[test]
    // fn test_archive_header_read_too_small() {
    //     assert!(ArchiveHeader::from_buf(&archive_header_prefix()).is_err());
    // }

    // #[test]
    // fn test_archive_header_prefix_too_small() {
    //     assert!(ArchiveHeader::from_buf(&[]).is_err())
    // }

    // #[test]
    // fn test_archive_header_parses() {
    //     assert!(ArchiveHeader::from_buf(&archive_header()).is_ok());
    // }

    // #[test]
    // fn test_archive_header_reads_reserved() {
    //     let buf = archive_header();
    //     let (head, _) = ArchiveHeader::from_buf(&buf).unwrap();
    //     assert_eq!(head.reserved1(), 1);
    //     assert_eq!(head.reserved2(), 2);
    // }
}
