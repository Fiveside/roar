mod archive;
mod file;
mod prefix;

pub use archive::ArchiveHeader;
pub use file::FilePrefix;
pub use prefix::BlockHeaderCommon;
pub use prefix::HeadType;

use crate::error::{Result, RoarError};
use crate::io::{ByteReader, CRC16Reader, FileReader};

#[derive(Debug)]
pub enum Block {
    Marker(archive::Marker),
    Archive(ArchiveHeader),
    File(FilePrefix),
}

pub async fn read_block(f: &mut impl FileReader) -> Result<Block> {
    let mut cursor = CRC16Reader::new(f);
    let block = BlockHeaderCommon::parse(&mut cursor).await?;

    Ok(match block.header_type {
        HeadType::MarkerBlock => Block::Marker(archive::Marker::parse(cursor, block).await),
        HeadType::ArchiveHeader => Block::Archive(ArchiveHeader::parse(cursor, block).await?),
        HeadType::FileHeader => Block::File(FilePrefix::parse(cursor, block).await?),
        _ => todo!("unimplemented block type: {:?}", block.header_type),
    })
}
