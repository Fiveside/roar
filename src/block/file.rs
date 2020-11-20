use nom::IResult;

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

pub fn file_header(_input: &[u8]) -> IResult<&[u8], FileHeader> {
    unimplemented!()
}
