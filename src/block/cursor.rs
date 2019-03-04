use crate::error::{Error, Result};
use byteorder::{ByteOrder, LittleEndian};
use crc::crc16;
use crc::crc16::Hasher16;
use tokio::io;
use tokio::prelude::*;

pub struct BufferCursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> BufferCursor<'a> {
    pub fn new(buf: &'a [u8]) -> BufferCursor<'a> {
        BufferCursor { buf: buf, pos: 0 }
    }

    pub fn read(&mut self, num: usize) -> Result<&'a [u8]> {
        if self.pos + num > self.buf.len() {
            return Err(Error::buffer_too_small(self.pos + num));
        }
        let ret = &self.buf[self.pos..self.pos + num];
        self.pos = self.pos + num;
        Ok(ret)
    }

    pub fn rest(self) -> &'a [u8] {
        &self.buf[self.pos..]
    }
}

pub struct AsyncCursor<'a, T: AsyncRead> {
    f: T,
    digest: &'a mut crc16::Digest,
}

impl<'a, T: AsyncRead> AsyncCursor<'a, T> {
    pub fn new(f: T, digest: &'a mut crc16::Digest) -> AsyncCursor<T> {
        AsyncCursor {
            f: f,
            digest: digest,
        }
    }

    pub async fn read_u8(&'a mut self) -> Result<u8> {
        let mut buf: [u8; 1] = unsafe { ::std::mem::uninitialized() };
        await!(io::read_exact(&mut self.f, &mut buf)).map_err(Error::io)?;
        self.digest.write(&buf);
        Ok(buf[0])
    }

    pub async fn read_u16(&'a mut self) -> Result<u16> {
        let mut buf: [u8; 2] = unsafe { ::std::mem::uninitialized() };
        await!(io::read_exact(&mut self.f, &mut buf)).map_err(Error::io)?;
        self.digest.write(&buf);
        Ok(LittleEndian::read_u16(&buf))
    }

    pub async fn read_u32(&'a mut self) -> Result<u32> {
        let mut buf: [u8; 4] = unsafe { ::std::mem::uninitialized() };
        await!(io::read_exact(&mut self.f, &mut buf)).map_err(Error::io)?;
        self.digest.write(&buf);
        Ok(LittleEndian::read_u32(&buf))
    }
}
