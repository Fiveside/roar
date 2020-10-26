// RAR3 Block parsing!
use std::fmt::Debug;

use nom::bytes::streaming::tag;
use nom::combinator::consumed;
use nom::number::streaming::{le_u16, le_u32, le_u8};
use nom::sequence::tuple;
use nom::IResult;

use crc::{crc32, Hasher32};

mod common;

// pub enum Block {
//     Marker(MarkerBlock),
//     Archive,
//     Unsupported,
// }

// pub struct MarkerBlock(BlockHeader);

// impl MarkerBlock {
//     fn header_crc_verified(&self) -> bool {
//         unimplemented!()
//     }
// }

// RAR3 magic = [82, 97, 114, 33, 26, 7, 0]
// RAR5 magic = [82, 97, 114, 33, 26, 7, 1, 0]

pub fn rar3_marker_block(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(&[82, 97, 114, 33, 26, 7, 0])(input)
}

pub fn rar5_marker_block(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(&[82, 97, 114, 33, 26, 7, 1, 0])(input)
}

pub struct ArchiveFlags(u16);

impl ArchiveFlags {
    fn is_volume(&self) -> bool {
        self.0 & 0x0001 != 0
    }

    fn is_comment_present(&self) -> bool {
        self.0 & 0x0002 != 0
    }

    fn is_locked(&self) -> bool {
        self.0 & 0x0004 != 0
    }

    fn is_solid(&self) -> bool {
        self.0 & 0x0008 != 0
    }

    fn is_new_volume_naming_scheme(&self) -> bool {
        // new naming scheme: volname.partN.rar
        self.0 & 0x0010 != 0
    }

    fn is_authenticity_info_present(&self) -> bool {
        self.0 & 0x0020 != 0
    }

    fn is_recovery_record_present(&self) -> bool {
        self.0 & 0x0040 != 0
    }

    fn is_block_headers_encrypted(&self) -> bool {
        self.0 & 0x0080 != 0
    }

    fn is_first_volume(&self) -> bool {
        self.0 & 0x0100 != 0
    }
}

impl Debug for ArchiveFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArchiveFlags")
            .field("raw_value", &self.0)
            .field("is_volume", &self.is_volume())
            .field("is_comment_present", &self.is_comment_present())
            .field("is_locked", &self.is_locked())
            .field("is_solid", &self.is_solid())
            .field(
                "is_new_volume_naming_scheme",
                &self.is_new_volume_naming_scheme(),
            )
            .field(
                "is_authenticity_info_present",
                &self.is_authenticity_info_present(),
            )
            .field(
                "is_recovery_record_present",
                &self.is_recovery_record_present(),
            )
            .field(
                "is_block_headers_encrypted",
                &self.is_block_headers_encrypted(),
            )
            .field("is_first_volume", &self.is_first_volume())
            .finish()
    }
}

#[derive(Debug)]
pub struct ArchiveHeader {
    head_crc: u16,
    head_type: u8,
    head_flags: ArchiveFlags,
    head_size: u16, // archive header total size including archive comments
    reserved1: u16,
    reserved2: u32,
}

pub fn archive_header(input: &[u8]) -> IResult<&[u8], ArchiveHeader> {
    let crc = le_u16;
    let marker = tag(&[0x73]);
    let content = tuple((marker, le_u16, le_u16, le_u16, le_u32));
    let (rest, (crc, (crc_buf, header_contents))) = tuple((crc, consumed(content)))(input)?;
    let (_typ, flags, size, res1, res2) = header_contents;

    let mut hasher = crc32::Digest::new(0xEDB88320);
    hasher.write(crc_buf);
    let computed_crc = hasher.sum32();
    println!(
        "ArchiveHeader expects crc {} got {} {}",
        crc,
        computed_crc,
        computed_crc & 0xffff
    );

    let ah = ArchiveHeader {
        head_crc: crc,
        head_type: 0x73,
        head_flags: ArchiveFlags(flags),
        head_size: size,
        reserved1: res1,
        reserved2: res2,
    };

    Ok((rest, ah))
}
