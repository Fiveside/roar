use super::BlockPrefix;
use crate::error::{Error, Result};
use crc::crc16;
use crc::crc16::Hasher16;

#[derive(Debug, Clone, Copy)]
pub struct ArchiveHeader<'a> {
    prefix: BlockPrefix<'a>,

    // RESERVED1 2 bytes
    // RESERVED2 4 bytes
    buf: &'a [u8],
}

impl<'a> ArchiveHeader<'a> {
    pub fn from(buf: &'a [u8]) -> Result<(ArchiveHeader<'a>, &'a [u8])> {
        let (prefix, prefix_rest) = BlockPrefix::from(buf)?;
        if prefix_rest.len() < 6 {
            return Err(Error::buffer_too_small(buf.len() - prefix_rest.len() + 6));
        }
        let rest = &prefix_rest[6..];

        return Ok((
            ArchiveHeader {
                prefix: prefix,
                buf: &prefix_rest[0..6],
            },
            rest,
        ));
    }

    pub fn prefix(&self) -> BlockPrefix<'a> {
        self.prefix
    }

    pub fn crc_digest(&self, seed: u16) -> crc16::Digest {
        let mut digest = self.prefix.crc_digest(seed);
        digest.write(self.buf);
        return digest;
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
        buf.extend(&[0, 0, 0, 0, 0, 0]);
        buf
    }

    #[test]
    fn test_archive_header_read_too_small() {
        assert!(ArchiveHeader::from(&archive_header_prefix()).is_err());
    }

    #[test]
    fn test_archive_header_prefix_too_small() {
        assert!(ArchiveHeader::from(&[]).is_err())
    }

    #[test]
    fn test_archive_header_parses() {
        assert!(ArchiveHeader::from(&archive_header()).is_ok());
    }
}
