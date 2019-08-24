mod archive;
mod cursor;
mod file;
mod prefix;

pub use archive::ArchiveHeader;
pub use file::FileHeader;
pub use prefix::BlockPrefix;
pub use prefix::HeadType;

use crate::error::{Error, Result};
use futures::io::{AsyncReadExt, AsyncRead};
use crate::traits::AsyncFile;

#[derive(Debug)]
pub enum Block {
    Marker,
    Archive(ArchiveHeader),
}


pub async fn read_block(f: &mut impl AsyncFile) -> Result<Block> {
//    let mut prefix_buf = ::std::mem::MaybeUninit::<[u8; 7]>::uninit();
//    unsafe {
//        io::read_exact(f, prefix_buf.as_mut_ptr()).await?;
//    }
//    let prefix_buf = unsafe { prefis_buf.assume_init() };
    let mut prefix_buf = [0; 7];
    f.read_exact(&mut prefix_buf).await?;
    let (block, rest) = BlockPrefix::from_buf(&prefix_buf)?;

    Ok(match block.block_type() {
        Some(prefix::HeadType::MarkerBlock) => Block::Marker,
        Some(HeadType::ArchiveHeader) => Block::Archive(ArchiveHeader::parse(block, f).await?),
        Some(_) => unimplemented!(),
        None => {
            return Err(Error::bad_block(format!(
                "Unknown block marker: {}",
                block.raw_block_type()
            )))
        }
    })
}
