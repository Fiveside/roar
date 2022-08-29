from http.client import UnimplementedFileMode
import struct
from pathlib import Path
from dataclasses import dataclass
import binascii

RAR3_MAGIC = bytes([82, 97, 114, 33, 26, 7, 0])
RAR5_MAGIC = bytes([82, 97, 114, 33, 26, 7, 1, 0])


@dataclass
class Rar3HeaderFlags:
    flags: int

    @property
    def uses_add_size(self):
        return self.flags & 0x8000

    @property
    def soft_deleted(self):
        return self.flags & 0x4000


@dataclass
class Rar3BlockBase:
    crc: int
    block_type: int
    flags: Rar3HeaderFlags
    size: int
    add_size: int
    rolling_crc: int

    @property
    def block_type_str(self):
        # HEAD_TYPE=0x72          marker block
        # HEAD_TYPE=0x73          archive header
        # HEAD_TYPE=0x74          file header
        # HEAD_TYPE=0x75          comment header
        # HEAD_TYPE=0x76          old style authenticity information
        # HEAD_TYPE=0x77          subblock
        # HEAD_TYPE=0x78          recovery record
        # HEAD_TYPE=0x79          authenticity information
        block_types = {
            0x72: "MagicMarker",
            0x73: "ArchiveHeader",
            0x74: "FileHeader",
            0x75: "CommentHeader",
            0x76: "LegacyAuthenticityRecord",
            0x77: "SubBlock",
            0x78: "Recovery",
            0x79: "AuthenticityRecord",
        }
        return block_types.get(self.block_type, f"Unknown({hex(self.block_type)})")

    @property
    def block_size(self):
        return (self.add_size << 16) + self.size


class Rar3Block:
    @staticmethod
    def parse_base(rario):
        # HEAD_CRC       2 bytes     CRC of total block or block part
        # HEAD_TYPE      1 byte      Block type
        # HEAD_FLAGS     2 bytes     Block flags
        # HEAD_SIZE      2 bytes     Block size
        # ADD_SIZE       4 bytes     Optional field - added block size
        crc = struct.unpack("<H", rario.read(2))[0]
        buf = rario.read(5)
        block_type, raw_flags, size = struct.unpack("<BHH", buf)
        flags = Rar3HeaderFlags(raw_flags)
        if flags.uses_add_size:
            add_buf = rario.read(4)
            buf += add_buf
            add_size = struct.unpack("<I", add_buf)
        else:
            add_size = (0,)

        rolling_crc = binascii.crc32(buf)
        return Rar3BlockBase(crc, block_type, flags, size, add_size, rolling_crc)

    @classmethod
    def parse(cls, rario):
        raise NotImplementedError()


class Rar3ArchiveHeader(Rar3Block):
    @dataclass
    class Rar3ArchiveHeaderFlags(Rar3HeaderFlags):
        # 0x01    - Volume attribute (archive volume)
        # 0x02    - Archive comment present
        # 0x04    - Archive lock attribute
        # 0x08    - Solid attribute (solid archive)
        # 0x10    - New volume naming scheme ('volname.partN.rar')
        # 0x20    - Authenticity information present
        # 0x40    - Recovery record present
        # 0x80    - Block headers are encrypted

        @property
        def is_volume(self):
            return self.flags & 0x01

        @property
        def archive_comment_present(self):
            return self.flags & 0x02

        @property
        def is_locked(self):
            return self.flags & 0x04

        @property
        def is_solid(self):
            return self.flags & 0x08

        @property
        def uses_new_volume_naming_scheme(self):
            return self.flags & 0x10

        @property
        def authenticity_information_present(self):
            return self.flags & 0x20

        @property
        def recovery_record_present(self):
            return self.flags & 0x40

        @property
        def block_headers_are_encrypted(self):
            return self.flags & 0x80

    def __init__(self, base, crc, reserved1, reserved2):
        self.base = base
        self.flags = self.Rar3ArchiveHeaderFlags(base.flags.flags)
        self.crc = crc
        self.reserved1 = reserved1
        self.reserved2 = reserved2

    @classmethod
    def parse(cls, base, rario):
        buf = rario.read(6)
        crc32 = binascii.crc32(buf, base.rolling_crc)
        return cls(base, crc32, *struct.unpack("<HI", buf))

        # block_types = {
        #     0x72: "MagicMarker",
        #     0x73: "ArchiveHeader",
        #     0x74: "FileHeader",
        #     0x75: "CommentHeader",
        #     0x76: "LegacyAuthenticityRecord",
        #     0x77: "SubBlock",
        #     0x78: "Recovery",
        #     0x79: "AuthenticityRecord",
        # }

    def is_valid(self):
        return self.crc & 0xFFFF == self.base.crc

    def __repr__(self):
        return f"ArchiveHeader(valid={self.is_valid()})"

class Rar3FileHeader(Rar3Block):
    @dataclass
    class Rar3FileHeaderFlags(Rar3HeaderFlags):
        @property
        def continued_from_previous_volume(self):
            return self.flags & 0x01

        @property
        def continues_in_next_volume(self):
            return self.flags & 0x02

        @property
        def encrypted_with_password(self):
            return self.flags & 0x04

        @property
        def comment_present(self):
            return self.flags & 0x08

        @property
        def is_member_of_solid_block(self):
            return self.flags & 0x10

        @property
        def dictionary_size(self):
            # Bits 7, 6, and 5 are used to determine this:
            bits = (
                bool(self.flags & 0x80),
                bool(self.flags & 0x40),
                bool(self.flags & 0x20)
            )
            dictmap = {
                (False, False, False): 64,
                (False, False, True): 128,
                (False, True, False): 256,
                (False, True, True): 512,
                (True, False, False): 1024,
                (True, False, True): 2048,
                (True, True, False): 4096,
                # (True, True, True): File is a dictionary
            }
            if bits in dictmap:
                return dictmap[bits]
            raise NotImplementedError("File is probably a dictionary")

        @property
        def high_fields_present(self):
            '''If this is true then the header uses HIGH_PACK_SIZE and HIGH_UNPACK_SIZE'''
            return self.flags & 0x100

        @property
        def has_unicode_packed_filename(self):
            return self.flags & 0x200

        @property
        def has_salt(self):
            return self.flags & 0x400

        @property
        def has_version_tag(self):
            return self.flags & 0x800

    def __init__(self, base, crc, file_crc):
        pass

    @classmethod
    def parse(cls, base, rario):
        flags = cls.Rar3FileHeaderFlags(base.flags.flags)


RAR3_BLOCK_TYPES = {0x73: Rar3ArchiveHeader}


def parse_block(rario):
    base = Rar3Block.parse_base(rario)
    klass = RAR3_BLOCK_TYPES[base.block_type]
    return klass.parse(base, rario)


def rar3file(rario):
    while True:
        block = parse_block(rario)
        import pdb

        pdb.set_trace()


def rar5file(rario):
    raise NotImplementedError()


def main(rario):
    try3buf = rario.read(len(RAR3_MAGIC))
    if try3buf == RAR3_MAGIC:
        rar3file(rario)
        return
    try5buf = try3buf + rario.read(len(RAR5_MAGIC) - len(RAR3_MAGIC))
    if try5buf == RAR5_MAGIC:
        rar5file(rario)
        return
    raise Exception("Cannot determine file type")


sample = Path(__file__).parent.joinpath(Path("./testcases/test.rar"))
with sample.open("rb") as fobj:
    main(fobj)
