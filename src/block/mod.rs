// RAR3 Block parsing!
use std::fmt::Debug;

use nom::bytes::streaming::tag;
use nom::combinator::consumed;
use nom::number::streaming::{le_u16, le_u32, le_u8};
use nom::sequence::tuple;
use nom::IResult;

use crc::{crc32, Hasher32};

mod common;

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

struct BlockPreamble {
    declared_header_crc: u16,
    flags: u16,
    header_size: u64,
    rolling_crc: crc32::Digest,
}

fn block_preamble(
    required_marker: u8,
) -> impl Fn(&[u8]) -> IResult<&[u8], &[u8], nom::error::ParseError> {
    move |input: &[u8]| {
        // let declared_crc = le_u16;
        // let marker = tag(&[required_marker]);
        // let flags = le_u16;
        // let head_size = le_u16;

        let (start_rest, (declared_crc, _, flags, low_head_size)) =
            tuple((le_u16, tag(&[required_marker]), le_u16, le_u16))(input)?;

        let (rest, header_size) = if flags & 0x8000 != 0 {
            let (end_rest, large_head_size) = le_u32(start_rest)?;
            (end_rest, low_head_size as u32 + large_head_size)
        } else {
            (start_rest, low_head_size as u32)
        };

        Ok((
            rest,
            BlockPreamble {
                declared_header_crc: declared_crc,
                flags: flags,
                header_size: header_size,
                rolling_crc: crc32::Digest::new(0xEDB88320),
            },
        ))
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

#[derive(Debug)]
pub struct FileFlags(u16);

impl FileFlags {
    fn is_continued_from_previous_volume(&self) -> bool {
        self.0 & 0x01 != 0
    }
    fn is_continued_in_next_volume(&self) -> bool {
        self.0 & 0x02 != 0
    }
    fn requires_password(&self) -> bool {
        self.0 & 0x04 != 0
    }
    fn file_comment_present(&self) -> bool {
        // rar version 3 and later use the separate comment block instead of
        // inline comments.
        self.0 & 0x08 != 0
    }
    fn uses_high_size_fields(&self) -> bool {
        // Look for high_pack_size and high_unpack_size if this is set
        self.0 & 0x100 != 0
    }
    fn has_unicode_filename(&self) -> bool {
        // Should trigger special filename parsing to look for the unicode
        // filename
        self.0 & 0x200 != 0
    }
    fn has_salt(&self) -> bool {
        self.0 & 0x400 != 0
    }
    fn file_is_versioned(&self) -> bool {
        self.0 & 0x800 != 0
    }
    fn has_ext_time(&self) -> bool {
        self.0 & 0x1000 != 0
    }
}

#[derive(Debug)]
pub struct FileHeader {
    head_crc: u16,
    head_type: u8,
    head_flags: FileFlags,
    // head_size: u16,
    // pack_size: u32,
    // unpack_size: u32,
    // host_os: u8, // enumify this
    // file_crc: u32,
    // ftime: u32,
    // unpack_version: u8, // Rar version needed to unpack this.
    // packing_method: u8, // enumify this
    // name_size: u16,
    // attrs: u32,  // file attributes
    // high_pack_size: u32,  // optional, todo: combine with pack size
    // high_unpack_size: u32, // optional, todo: combine with unpack size
    // file_name: vec![], // requires reference to name_size
    // salt: [u8; 8], // optional, crypto salt
    // ext_time: vec![] // optional, ext_time itself is variadic.
}

pub fn file_header(input: &[u8]) -> IResult<&[u8], FileHeader> {}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn preamble_parse_matches_required_tag() {
        let buf = RAR3_MAGIC;
        let (rest, _) = block_preamble(0x73)(&buf);
        assert_eq!(rest, &[]);
    }
}
