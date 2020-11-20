use std::fmt::Debug;

use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::combinator::{consumed, map, map_opt, not, recognize, verify};
use nom::number::streaming::{le_u16, le_u32, le_u8};
use nom::sequence::{pair, tuple};
use nom::IResult;

use crc::{crc32, Hasher32};

pub struct BlockPreamble {
    // checksum as declared in the block header
    pub declared_header_crc: u16,

    // Flags as declared
    pub flags: u16,

    // Header size as declared minus bytes required for this block preamble
    pub remaining_header_size: u32,

    // A digest seeded with all data consumed while parsing this block so far
    pub rolling_crc: crc32::Digest,
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

pub fn block_preamble(required_marker: u8) -> impl Fn(&[u8]) -> IResult<&[u8], BlockPreamble> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn preamble_parse_matches_required_tag() {
        let buf = [82, 97, 114, 33, 26, 7, 0];
        let (rest, _) = block_preamble(0x72)(&buf).unwrap();
        assert_eq!(rest, &[]);
    }

    #[test]
    fn preamble_parse_rejects_bad_tags() {
        let buf = [82, 97, 114, 33, 26, 7, 0];
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
        let buf = [82, 97, 114, 33, 26, 7, 0];
        let (_rest, res) = block_preamble(0x72)(&buf).unwrap();
        assert_eq!(res.rolling_crc.sum32(), 803352714);
    }
}
