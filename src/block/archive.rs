use self::prefix::BlockPrefix;

pub struct ArchiveHeader<'a> {
    prefix: 'a BlockPrefix,

    // RESERVED1
    // RESERVED2
    buf: &'a [u8],
}
