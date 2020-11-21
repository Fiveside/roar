use std::fmt::Debug;

use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::combinator::{consumed, map, map_opt, not, recognize, verify};
use nom::number::streaming::{le_u16, le_u32, le_u8};
use nom::sequence::{pair, tuple};
use nom::IResult;

use crc::{crc32, Hasher32};

use super::preamble::{block_preamble, BlockPreamble};

#[derive(Debug)]
pub struct CrcResult {
    expected: u16,
    actual: u32,
}

impl CrcResult {
    pub fn new(expected: u16, actual: u32) -> Self {
        Self { expected, actual }
    }
    pub fn from_preamble(buf: &[u8], preamble: BlockPreamble) -> Self {
        let expected = preamble.declared_header_crc;
        let mut hasher = preamble.rolling_crc;
        hasher.write(buf);
        let actual = hasher.sum32();
        Self { expected, actual }
    }
}

#[derive(Debug)]
pub struct ArchiveHeader {
    crc: CrcResult,
    head_flags: ArchiveFlags,
}

pub fn archive_header(input: &[u8]) -> IResult<&[u8], ArchiveHeader> {
    // let crc = le_u16;
    // let marker = tag(&[0x73]);
    // let content = tuple((marker, le_u16, le_u16, le_u16, le_u32));
    // let (rest, (crc, (crc_buf, header_contents))) = tuple((crc, consumed(content)))(input)?;
    // let (_typ, flags, size, res1, res2) = header_contents;

    let preamble = block_preamble(0x73);
    let reserved = recognize(pair(le_u16, le_u32));
    let verified_preamble = verify(preamble, |p| p.remaining_header_size == 6);

    let (rest, (preamble, reserved_buf)) = pair(verified_preamble, reserved)(input)?;

    let BlockPreamble {
        declared_header_crc,
        flags,
        rolling_crc: mut hasher,
        ..
    } = preamble;

    hasher.write(reserved_buf);
    let actual_crc = hasher.sum32();

    // let mut hasher = crc32::Digest::new(0xEDB88320);
    // hasher.write(crc_buf);
    // let computed_crc = hasher.sum32();
    // println!(
    //     "ArchiveHeader expects crc {} got {} {}",
    //     crc,
    //     computed_crc,
    //     computed_crc & 0xffff
    // );

    let ah = ArchiveHeader {
        crc: CrcResult {
            expected: declared_header_crc,
            actual: actual_crc,
        },
        head_flags: ArchiveFlags(flags),
    };

    Ok((rest, ah))
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
        // old naming scheme: volname.r00
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
