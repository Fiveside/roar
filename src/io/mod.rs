use crate::error::Result;
use async_trait::async_trait;
use byteorder::{ByteOrder, LittleEndian};
use futures::io::{AsyncRead, AsyncReadExt, AsyncSeek, SeekFrom};
use futures::AsyncSeekExt;
use std::marker::Unpin;
use crc::crc16;
use crc::crc16::Hasher16;

#[async_trait]
pub trait FileReader: Unpin + Send {
    async fn read(&mut self, amount: usize) -> Result<Vec<u8>>;
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
}

pub struct AsyncFileReader<T: AsyncRead + AsyncSeek + Unpin + Send> {
    f: T,
}

impl<T: AsyncRead + AsyncSeek + Unpin + Send> AsyncFileReader<T> {
    pub fn new(f: T) -> Self {
        AsyncFileReader { f }
    }
}

#[async_trait]
impl<T: AsyncRead + AsyncSeek + Unpin + Send> FileReader for AsyncFileReader<T> {
    async fn read(&mut self, amount: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; amount];
        self.f.read_exact(&mut buf).await?;
        Ok(buf)
    }

    async fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        Ok(self.f.seek(pos).await?)
    }
}

pub struct ByteReader<'a, T: FileReader> {
    f: &'a mut T,
}

impl<'a, T: FileReader> ByteReader<'a, T> {
    pub fn new(f: &'a mut T) -> Self {
        ByteReader { f }
    }

    pub async fn read(&mut self, amount: usize) -> Result<Vec<u8>> {
        Ok(self.f.read(amount).await?)
    }

    pub async fn read_u8(&mut self) -> Result<u8> {
        let res = self.f.read(1).await?;
        Ok(res[0])
    }

    pub async fn read_u16(&mut self) -> Result<u16> {
        let res = self.f.read(2).await?;
        Ok(LittleEndian::read_u16(&res))
    }

    pub async fn read_u32(&mut self) -> Result<u32> {
        let res = self.f.read(4).await?;
        Ok(LittleEndian::read_u32(&res))
    }

    pub async fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.f.seek(pos).await
    }
}

pub struct CRC16Reader<'a, T: FileReader> {
    f: &'a mut T,
    _hasher: crc16::Digest,
}

impl<'a, T: FileReader> CRC16Reader<'a, T> {
    pub fn new(f: &'a mut T) -> Self {
        todo!("Hasher digest of 0 is not correct.  Figure out the correct one.");
        let hasher = crc::crc16::Digest::new(0);
        CRC16Reader {f, _hasher: hasher}
    }

    pub fn resume(f: &'a mut T, hasher: crc16::Digest) -> Self {
        CRC16Reader {f, _hasher: hasher}
    }

    pub fn hasher(self) -> crc16::Digest {
        self._hasher
    }

    pub async fn read(&mut self, amount: usize) -> Result<Vec<u8>> {
        let res = self.f.read(amount).await?;
        self._hasher.write(&res);
        Ok(res)
    }

    pub async fn read_u8(&mut self) -> Result<u8> {
        let res = self.f.read(1).await?;
        self._hasher.write(&res);
        Ok(res[0])
    }

    pub async fn read_u16(&mut self) -> Result<u16> {
        let res = self.f.read(2).await?;
        self._hasher.write(&res);
        Ok(LittleEndian::read_u16(&res))
    }

    pub async fn read_u32(&mut self) -> Result<u32> {
        let res = self.f.read(4).await?;
        self._hasher.write(&res);
        Ok(LittleEndian::read_u32(&res))
    }

    pub async fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.f.seek(pos).await
    }
}