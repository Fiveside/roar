mod archive;
//mod file;
mod prefix;

pub use archive::ArchiveHeader;
//pub use file::FileHeader;
pub use prefix::BlockHeaderCommon;
pub use prefix::HeadType;

use crate::error::{RoarError, Result};
use crate::io::{ByteReader, FileReader, CRC16Reader};

#[derive(Debug)]
pub enum Block {
    Marker(archive::Marker),
    Archive(ArchiveHeader),
}

pub async fn read_block(f: &mut impl FileReader) -> Result<Block> {
    //    let mut prefix_buf = ::std::mem::MaybeUninit::<[u8; 7]>::uninit();
    //    unsafe {
    //        io::read_exact(f, prefix_buf.as_mut_ptr()).await?;
    //    }
    //    let prefix_buf = unsafe { prefis_buf.assume_init() };
    let mut cursor = CRC16Reader::new(f);
    let block = BlockHeaderCommon::read_from_file(&mut cursor).await?;

    Ok(match block.header_type {
        HeadType::MarkerBlock => Block::Marker(archive::Marker::parse(cursor, block).await),
        HeadType::ArchiveHeader => Block::Archive(ArchiveHeader::parse(cursor, block).await?),
        _ => { todo!() }
    })
}
