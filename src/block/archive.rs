use super::cursor::BufferCursor;
use super::BlockPrefix;
use crate::error::Result;
use byteorder::{ByteOrder, LittleEndian};
use crc::crc16;
use crc::crc16::Hasher16;

#[derive(Debug, Clone, Copy)]
pub struct ArchiveHeader<'a> {
    prefix: BlockPrefix<'a>,

    // HighPosAv 2 bytes
    // PosAv 4 bytes
    // Optional 1 byte EncryptVer (not implemented right now)
    buf: &'a [u8],
}

impl<'a> ArchiveHeader<'a> {
    pub fn from_buf(buf: &'a [u8]) -> Result<(ArchiveHeader<'a>, &'a [u8])> {
        let mut cursor = BufferCursor::new(buf);
        let ah = ArchiveHeader::from_cursor(&mut cursor)?;
        Ok((ah, cursor.rest()))
    }

    pub fn from_cursor(cursor: &mut BufferCursor<'a>) -> Result<ArchiveHeader<'a>> {
        let prefix = BlockPrefix::from_cursor(cursor)?;
        Ok(ArchiveHeader {
            prefix: prefix,
            buf: cursor.read(6)?,
        })
    }

    pub fn prefix(&self) -> BlockPrefix<'a> {
        self.prefix
    }

    pub fn crc_digest(&self, seed: u16) -> crc16::Digest {
        let mut digest = self.prefix.crc_digest(seed);
        digest.write(self.buf);
        return digest;
    }

    pub fn reserved1(&self) -> u16 {
        LittleEndian::read_u16(self.buf)
    }

    pub fn reserved2(&self) -> u32 {
        LittleEndian::read_u32(&self.buf[2..])
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

    #[test]
    fn test_archive_header_read_too_small() {
        assert!(ArchiveHeader::from_buf(&archive_header_prefix()).is_err());
    }

    #[test]
    fn test_archive_header_prefix_too_small() {
        assert!(ArchiveHeader::from_buf(&[]).is_err())
    }

    #[test]
    fn test_archive_header_parses() {
        assert!(ArchiveHeader::from_buf(&archive_header()).is_ok());
    }

    #[test]
    fn test_archive_header_reads_reserved() {
        let buf = archive_header();
        let (head, _) = ArchiveHeader::from_buf(&buf).unwrap();
        assert_eq!(head.reserved1(), 1);
        assert_eq!(head.reserved2(), 2);
    }
}
