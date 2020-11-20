// RAR3 Block parsing!
use std::fmt::Debug;

use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::combinator::{consumed, map, map_opt, not, recognize, verify};
use nom::number::streaming::{le_u16, le_u32, le_u8};
use nom::sequence::{pair, tuple};
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

#[derive(Debug)]
pub struct CrcResult {
    expected: u16,
    actual: u32,
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
    // checksum as declared in the block header
    declared_header_crc: u16,

    // Flags as declared
    flags: u16,

    // Header size as declared minus bytes required for this block preamble
    remaining_header_size: u32,

    // A digest seeded with all data consumed while parsing this block so far
    rolling_crc: crc32::Digest,
}

impl Debug for BlockPreamble {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockPreamble")
            .field("declared_header_crc", &self.declared_header_crc)
            .field("flags", &self.flags)
            .field("remaining_header_size", &self.remaining_header_size)
            .field("rolling_crc", &self.rolling_crc.sum32())
            .finish()
    }
}

fn block_preamble_opening(
    required_marker: u8,
) -> impl Fn(&[u8]) -> IResult<&[u8], (u16, u16, u32)> {
    move |input: &[u8]| {
        // let (start_rest, (declared_crc, _, flags, low_head_size)) =
        //     tuple((le_u16, tag(&[required_marker] as &[u8]), le_u16, le_u16))(input)?;

        let head_size = map(le_u16, |x| x as u32);
        let add_size = le_u32;

        // If we only have head size, then the size of the preamble is 7 bytes
        let just_head_size = map_opt(le_u16, |x| (x as u32).checked_sub(7));

        // If we have both head size and add size, then the size of the
        // preamble is 11 bytes.
        let both_size = map_opt(pair(head_size, add_size), |(l, r)| {
            l.checked_add(r).and_then(|x| x.checked_sub(11))
        });

        let has_add_size = verify(le_u16, |x| x & 0x8000 != 0);
        let no_add_size = verify(le_u16, |x| x & 0x8000 == 0);

        let flags_and_size = alt((
            pair(has_add_size, both_size),
            pair(no_add_size, just_head_size),
        ));

        let block_type_binding = [required_marker];
        let block_type = tag(&block_type_binding);
        let crc = le_u16;
        let (rest, (declared_crc, _block_type, (flags, size))) =
            tuple((crc, block_type, flags_and_size))(input)?;

        Ok((rest, (declared_crc, flags, size)))
    }
}

fn block_preamble(required_marker: u8) -> impl Fn(&[u8]) -> IResult<&[u8], BlockPreamble> {
    move |input: &[u8]| {
        let (rest, (consumed_buf, (declared_crc, flags, adjusted_head_size))) =
            consumed(block_preamble_opening(required_marker))(input)?;

        let mut digest = crc32::Digest::new(0xEDB88320);
        digest.write(consumed_buf);

        Ok((
            rest,
            BlockPreamble {
                declared_header_crc: declared_crc,
                flags: flags,
                remaining_header_size: adjusted_head_size,
                rolling_crc: digest,
            },
        ))
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

pub fn file_header(input: &[u8]) -> IResult<&[u8], FileHeader> {
    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn preamble_parse_matches_required_tag() {
        let buf = RAR3_MAGIC;
        let (rest, _) = block_preamble(0x72)(&buf).unwrap();
        assert_eq!(rest, &[]);
    }

    #[test]
    fn preamble_parse_rejects_bad_tags() {
        let buf = RAR3_MAGIC;
        let res = block_preamble(0x73)(&buf);
        assert!(res.is_err());
    }

    #[test]
    fn preamble_looks_for_add_size() {
        let flags_high: u8 = 0x80;
        let flags_low = 0x00;
        let buf = [82, 97, 114, flags_low, flags_high, 12, 0, 2, 0, 0, 0];
        let (rest, res) = block_preamble(0x72)(&buf).unwrap();
        assert_eq!(rest, &[]);
        assert_eq!(res.remaining_header_size, 3);
    }

    #[test]
    fn preamble_ignores_add_size() {
        let flags_high: u8 = 0x00;
        let flags_low = 0x00;
        let buf = [82, 97, 114, flags_low, flags_high, 8, 0];
        let (rest, res) = block_preamble(0x72)(&buf).unwrap();
        assert_eq!(rest, &[]);
        assert_eq!(res.remaining_header_size, 1);
    }

    #[test]
    fn preamble_header_size_underflow_returns_error() {
        let flags_high: u8 = 0x00;
        let flags_low: u8 = 0x00;
        for size in 0..7 {
            let buf = [82, 97, 114, flags_low, flags_high, size, 0];
            let res = block_preamble(0x72)(&buf);
            assert!(
                res.is_err(),
                format!("size of {} did not return error", size)
            );
        }
    }

    #[test]
    fn preamble_add_size_underflow_returns_error() {
        let flags_high: u8 = 0x00;
        let flags_low: u8 = 0x00;
        for size in 0..11 {
            let buf = [82, 97, 114, flags_low, flags_high, 0, 0, size, 0, 0, 0];
            let res = block_preamble(0x72)(&buf);
            assert!(
                res.is_err(),
                format!("size of {} did not return error", size)
            );
        }
    }

    #[test]
    fn preamble_digest_is_seeded() {
        let buf = RAR3_MAGIC;
        let (_rest, res) = block_preamble(0x72)(&buf).unwrap();
        assert_eq!(res.rolling_crc.sum32(), 803352714);
    }
}
