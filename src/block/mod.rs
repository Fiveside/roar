// RAR3 Block parsing!
use std::fmt::Debug;

use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::combinator::{consumed, map, map_opt, not, recognize, value, verify};
use nom::number::streaming::{le_u16, le_u32, le_u8};
use nom::sequence::{pair, tuple};
use nom::IResult;

use crc::{crc32, Hasher32};

mod archive;
mod common;
mod file;
mod preamble;

// HEAD_TYPE=0x72          marker block
// HEAD_TYPE=0x73          archive header
// HEAD_TYPE=0x74          file header
// HEAD_TYPE=0x75          old style comment header
// HEAD_TYPE=0x76          old style authenticity information
// HEAD_TYPE=0x77          old style subblock
// HEAD_TYPE=0x78          old style recovery record
// HEAD_TYPE=0x79          old style authenticity information
// HEAD_TYPE=0x7a          subblock

const RAR3_MAGIC: [u8; 7] = [82, 97, 114, 33, 26, 7, 0];
const RAR5_MAGIC: [u8; 8] = [82, 97, 114, 33, 26, 7, 1, 0];

pub fn rar3_marker_block(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(&RAR3_MAGIC)(input)
}

pub fn rar5_marker_block(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(&RAR5_MAGIC)(input)
}

#[derive(Debug)]
pub enum BlockHeader {
    Marker,
    Archive(archive::ArchiveHeader),
    File(file::FileHeader),
}

pub fn block_header(input: &[u8]) -> IResult<&[u8], BlockHeader> {
    // let marker = value(BlockHeader::Marker, rar3_marker_block);
    let marker = map(rar3_marker_block, |_| BlockHeader::Marker);
    let archive = map(archive::archive_header, |x| BlockHeader::Archive(x));
    let file = map(file::file_header, |x| BlockHeader::File(x));

    alt((marker, archive, file))(input)
}
