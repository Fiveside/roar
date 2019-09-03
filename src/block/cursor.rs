use crate::error::{Error, Result};
use crate::traits::AsyncFile;
use async_std::io::Read;
use byteorder::{ByteOrder, LittleEndian};
use crc::crc16;
use crc::crc16::Hasher16;
use futures::io::{AsyncRead, AsyncReadExt};
use std::mem::MaybeUninit;
//use async_std::io::Read;

pub struct BufferCursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> BufferCursor<'a> {
    pub fn new(buf: &'a [u8]) -> BufferCursor<'a> {
        BufferCursor { buf, pos: 0 }
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

pub struct AsyncCRC16Cursor<T: AsyncRead + Unpin> {
    pub file: T,
    pub digest: crc16::Digest,
}

impl<T: AsyncRead + Unpin> AsyncCRC16Cursor<T> {
    pub fn new_with_digest(f: T, digest: crc::crc16::Digest) -> AsyncCRC16Cursor<T> {
        AsyncCRC16Cursor { file: f, digest }
    }

    pub fn new(f: T, digest_seed: u16) -> AsyncCRC16Cursor<T> {
        AsyncCRC16Cursor {
            file: f,
            digest: crc::crc16::Digest::new(digest_seed),
        }
    }

    pub async fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        async_std::io::Read::read_exact(&mut self.file, &mut buf).await?;
//        self.file.read_exact(&mut buf).await?;
        self.digest.write(&buf);
        Ok(buf[0])
    }

    pub async fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        async_std::io::Read::read_exact(&mut self.file, &mut buf).await?;
//        self.file.read_exact(&mut buf).await?;
        self.digest.write(&buf);
        Ok(LittleEndian::read_u16(&buf))
    }

    pub async fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        async_std::io::Read::read_exact(&mut self.file, &mut buf).await?;
//        self.file.read_exact(&mut buf).await?;
        self.digest.write(&buf);
        Ok(LittleEndian::read_u32(&buf))
    }
}

//pub struct AsyncCursor<'a, T: AsyncFile> {
//    f: T,
//    digest: &'a mut crc16::Digest,
//}
//
//impl<'a, T: AsyncFile> AsyncCursor<'a, T> {
//    pub fn new(f: T, digest: &'a mut crc16::Digest) -> AsyncCursor<T> {
//        AsyncCursor { f, digest }
//    }
//
//    pub async fn read_u8(&'a mut self) -> Result<u8> {
//        //        let mut buf = MaybeUninit::<[u8; 1]>::uninit();
//        //        unsafe {
//        //            self.f.read_exact(&*buf.as_mut_ptr()).await?;
//        //        }
//        //        let buf = unsafe { buf.assume_init() };
//        //        self.digest.write(&buf);
//        //        Ok(buf[0])
//        let mut buf = [0; 1];
//        self.f.read_exact(&mut buf).await?;
//        self.digest.write(&buf);
//        Ok(buf[0])
//    }
//
//    pub async fn read_u16(&'a mut self) -> Result<u16> {
//        //        let mut buf = MaybeUninit::<[u8; 2]>::uninit();
//        //        unsafe {
//        //            io::read_exact(&mut self.f, buf.as_mut_ptr()).await?;
//        //        }
//        //        let buf = unsafe { buf.assume_init() };
//        //        self.digest.write(&buf);
//        //        Ok(LittleEndian::read_u16(&buf))
//        let mut buf = [0; 2];
//        self.f.read_exact(&mut buf).await?;
//        self.digest.write(&buf);
//        Ok(LittleEndian::read_u16(&buf))
//    }
//
//    pub async fn read_u32(&'a mut self) -> Result<u32> {
//        //        let mut buf = MaybeUninit::<[u8; 4]>::uninit();
//        //        unsafe {
//        //            io::read_exact(&mut self.f, buf.as_mut_ptr()).await?;
//        //        }
//        //        let buf = unsafe { buf.assume_init() };
//        //        self.digest.write(&buf);
//        //        Ok(LittleEndian::read_u32(&buf))
//        let mut buf = [0; 4];
//        self.f.read_exact(&mut buf).await?;
//        self.digest.write(&buf);
//        Ok(LittleEndian::read_u32(&buf))
//    }
//}
