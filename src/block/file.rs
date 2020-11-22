use super::archive::CrcResult;
use super::preamble::{block_preamble, BlockPreamble};

use nom::branch::alt;
use nom::bytes::streaming::{tag, take, take_while};
use nom::combinator::{cond, map, map_parser, not, rest, verify};
use nom::multi::length_data;
use nom::number::streaming::{le_u16, le_u32, le_u8};
use nom::sequence::{preceded, separated_pair, terminated, tuple};
use nom::IResult;

use crc::Hasher32;

#[derive(Debug, Copy, Clone)]
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
    header_crc: CrcResult,
    head_flags: FileFlags,
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

#[derive(Debug)]
struct FileName {
    non_unicode: Vec<u8>,
    unicode: Option<String>,
}

impl FileName {
    fn new_non_unicode(name_data: &[u8]) -> Self {
        Self {
            non_unicode: name_data.into(),
            unicode: None,
        }
    }
    fn new_unicode(name_data: &[u8], unicode_name_data: String) -> Self {
        Self {
            non_unicode: name_data.into(),
            unicode: Some(unicode_name_data),
        }
    }
}

fn pack_size(input: &[u8]) -> IResult<&[u8], u32> {
    le_u32(input)
}

fn unpack_size(input: &[u8]) -> IResult<&[u8], u32> {
    le_u32(input)
}

fn host_os(input: &[u8]) -> IResult<&[u8], u8> {
    le_u8(input)
}

fn file_crc(input: &[u8]) -> IResult<&[u8], u32> {
    le_u32(input)
}

fn ftime(input: &[u8]) -> IResult<&[u8], u32> {
    le_u32(input)
}

fn unpack_version(input: &[u8]) -> IResult<&[u8], u8> {
    le_u8(input)
}

fn packing_method(input: &[u8]) -> IResult<&[u8], u8> {
    le_u8(input)
}

fn name_size(input: &[u8]) -> IResult<&[u8], u16> {
    le_u16(input)
}

fn attrs(input: &[u8]) -> IResult<&[u8], u32> {
    le_u32(input)
}

fn high_pack_size(flags: FileFlags) -> impl Fn(&[u8]) -> IResult<&[u8], u32> {
    move |input: &[u8]| {
        map(cond(flags.uses_high_size_fields(), le_u32), |x| {
            x.unwrap_or(0)
        })(input)
    }
}

fn high_unpack_size(flags: FileFlags) -> impl Fn(&[u8]) -> IResult<&[u8], u32> {
    move |input: &[u8]| {
        map(cond(flags.uses_high_size_fields(), le_u32), |x| {
            x.unwrap_or(0)
        })(input)
    }
}

fn is_null(test: u8) -> bool {
    test == 0
}

fn is_not_null(test: u8) -> bool {
    !is_null(test)
}

fn packed_unicode_filename(input: &[u8]) -> IResult<&[u8], FileName> {
    let non_unicode_parser = take_while(is_not_null);
    let sep = tag(&[0]);
    let unicode_parser = rest;
    let (rest, (non_unicode, unicode_bytes)) =
        separated_pair(non_unicode_parser, sep, unicode_parser)(input)?;

    // TODO: unsafe unwrap
    let unicode = std::str::from_utf8(unicode_bytes).unwrap().to_owned();

    Ok((rest, FileName::new_unicode(non_unicode, unicode)))
}

fn raw_filename_buffer(flags: FileFlags) -> impl FnMut(&[u8]) -> IResult<&[u8], &[u8]> {
    move |input: &[u8]| {
        let skip1 = tuple((
            pack_size,
            unpack_size,
            host_os,
            file_crc,
            ftime,
            unpack_version,
            packing_method,
        ));

        let skip2 = tuple((attrs, high_pack_size(flags), high_unpack_size(flags)));
        let size = terminated(preceded(skip1, le_u16), skip2);
        length_data(size)(input)
    }
}

fn read_file_name(input: &[u8], flags: FileFlags) -> IResult<&[u8], FileName> {
    let non_unicode_name = cond(
        !flags.has_unicode_filename(),
        map(raw_filename_buffer(flags), |x| FileName::new_non_unicode(x)),
    );
    let unicode_name = cond(
        flags.has_unicode_filename(),
        map_parser(raw_filename_buffer(flags), packed_unicode_filename),
    );
    let (rest, filename_opt) = alt((non_unicode_name, unicode_name))(input)?;

    // Safe unwrap, the above alt hits 2 opposite conditions.  One must resolve.
    Ok((rest, filename_opt.unwrap()))
}

pub fn file_header(input: &[u8]) -> IResult<&[u8], FileHeader> {
    let (most_rest, preamble) = block_preamble(0x74)(input)?;
    let (rest, file_buf) = take(preamble.remaining_header_size)(most_rest)?;
    let flags = FileFlags(preamble.flags);
    let header_crc = CrcResult::from_preamble(file_buf, preamble);

    Ok((
        rest,
        FileHeader {
            header_crc,
            head_flags: flags,
        },
    ))
}
