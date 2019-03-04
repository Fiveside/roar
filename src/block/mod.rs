mod archive;
mod cursor;
mod file;
mod prefix;

pub use archive::ArchiveHeader;
pub use file::FileHeader;
pub use prefix::BlockPrefix;
pub use prefix::HeadType;

use crate::error::{Error, Result};
use tokio::io;

#[derive(Debug)]
pub enum Block {
    Marker,
    Archive(ArchiveHeader),
}

pub async fn read_block(f: &mut impl io::AsyncRead) -> Result<Block> {
    let mut prefix_buf: [u8; 7] = unsafe { ::std::mem::uninitialized() };
    await!(io::read_exact(f, &mut prefix_buf)).map_err(Error::io)?;

    let (block, rest) = BlockPrefix::from_buf(&prefix_buf)?;
    Ok(match block.block_type() {
        Some(prefix::HeadType::MarkerBlock) => Block::Marker,
        Some(HeadType::ArchiveHeader) => Block::Archive(await!(ArchiveHeader::parse(&block, f))?),
        Some(_) => unimplemented!(),
        None => {
            return Err(Error::bad_block(format!(
                "Unknown block marker: {}",
                block.raw_block_type()
            )))
        }
    })
}
