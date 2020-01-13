use crate::block::prefix::BlockHeaderCommon;
use crate::error::Result;
use crate::io::{CRC16Reader, FileReader};
use byteorder::{ByteOrder, LittleEndian};
use crc::crc16;
use crc::crc16::Hasher16;
use futures::io::AsyncRead;
use std::hash::Hasher;

#[derive(Debug)]
pub struct Marker {
    pub prefix: BlockHeaderCommon,
    pub crc: u16,
}

impl Marker {
    pub async fn parse<T: FileReader>(f: CRC16Reader<'_, T>, prefix: BlockHeaderCommon) -> Self {
        Marker {
            prefix,
            crc: f.hasher().sum16(),
        }
    }
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
    pub async fn parse<T: FileReader>(
        mut f: CRC16Reader<'_, T>,
        prefix: BlockHeaderCommon,
    ) -> Result<ArchiveHeader> {
        // FIXME: the digest seed is incorrect.
        let reserved1 = f.read_u16().await?;
        let reserved2 = f.read_u32().await?;
        let block_crc = f.hasher().sum16();
        Ok(ArchiveHeader {
            prefix,
            block_crc,
            reserved1,
            reserved2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::io::{Cursor, Read, Seek};

    fn archive_header_prefix() -> Vec<u8> {
        vec![207, 144, 115, 0, 0, 13, 0]
    }

    fn archive_header() -> Vec<u8> {
        let mut buf = archive_header_prefix();
        buf.extend(&[1, 0, 2, 0, 0, 0]);
        buf
    }

    struct BufReader {
        cursor: Cursor<Vec<u8>>,
    }

    #[async_trait]
    impl FileReader for BufReader {
        async fn read(&mut self, amount: usize) -> Result<Vec<u8>> {
            let mut buf = vec![0; amount];
            self.cursor.read_exact(&mut buf).unwrap();
            Ok(buf)
        }

        async fn seek(&mut self, pos: ::futures::io::SeekFrom) -> Result<u64> {
            Ok(self.cursor.seek(pos)?)
        }
    }

    //    fn reader(buf: Vec<u8>) -> CRC16Reader<'_, BufReader> {
    //        let cursor = Cursor::new(buf);
    //        let reader =
    //        CRC16Reader::new(BufReader{ cursor })
    //    }

    //    macro_rules! reader {
    //        ( $x:expr ) => {{
    //            let mut bufreader = BufReader { cursor: Cursor::new($x) };
    //            return CRC16Reader::new(&mut bufreader)
    //        }}
    //    }

    #[async_std::test]
    async fn test_archive_header_read_too_small() -> Result<()> {
        let mut bufreader = BufReader {
            cursor: Cursor::new(archive_header()),
        };
        let mut f = CRC16Reader::new(&mut bufreader);
        let prefix = BlockHeaderCommon::parse(&mut f).await?;
        let ah = ArchiveHeader::parse(f, prefix).await?;
        assert_eq!(ah.block_crc, ah.prefix.expected_header_crc);

        Ok(())
    }

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
