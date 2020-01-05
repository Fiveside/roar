mod archive;
mod cursor;
//mod file;
mod prefix;

pub use archive::ArchiveHeader;
//pub use file::FileHeader;
pub use prefix::BlockHeaderCommon;
pub use prefix::HeadType;

use crate::error::{Error, Result};
use futures::io::{AsyncRead, AsyncReadExt};

#[derive(Debug)]
pub enum Block {
    Marker,
    Archive(ArchiveHeader),
}

pub async fn read_block<T: AsyncRead>(f: &mut T) -> Result<Block> {
    //    let mut prefix_buf = ::std::mem::MaybeUninit::<[u8; 7]>::uninit();
    //    unsafe {
    //        io::read_exact(f, prefix_buf.as_mut_ptr()).await?;
    //    }
    //    let prefix_buf = unsafe { prefis_buf.assume_init() };
    let block = BlockHeaderCommon::read_from_file(f).await?;

    Ok(match block.header_type {
        HeadType::MarkerBlock => Block::Marker,
        HeadType::ArchiveHeader => Block::Archive(ArchiveHeader::parse(block, f).await?),
        _ => {
            return Err(Error::bad_block(format!(
                "Unknown block marker: {:?}",
                block.header_type
            )))
        }
    })
}
