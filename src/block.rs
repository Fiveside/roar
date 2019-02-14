#[derive(FromPrimitive)]
enum HeadType {
    MarkerBlock = 0x72,
    ArchiveHeader = 0x73,
    FileHeader = 0x74,
    OldCommentHeader = 0x75,
    OldAuthenticityInformation = 0x76,
    OldSubBlock = 0x77,
    OldRecoveryRecord = 0x78,
    OldAuthenticityInformation2 = 0x79,
    SubBlock = 0x7a,
    Terminator = 0x7b,
    Unknown,
}
