use crate::error::Result;
use async_trait::async_trait;
use byteorder::{ByteOrder, LittleEndian};
use futures::io::{AsyncRead, AsyncReadExt};
use std::marker::Unpin;

#[async_trait]
pub trait FileReader: Unpin + Send {
    async fn read(&mut self, amount: usize) -> Result<Vec<u8>>;
}

pub struct AsyncFileReader<T: AsyncRead + Unpin + Send> {
    f: T,
}

impl<T: AsyncRead + Unpin + Send> AsyncFileReader<T> {
    pub fn new(f: T) -> Self {
        AsyncFileReader { f }
    }
}

#[async_trait]
impl<T: AsyncRead + Unpin + Send> FileReader for AsyncFileReader<T> {
    async fn read(&mut self, amount: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; amount];
        self.f.read_exact(&mut buf).await?;
        Ok(buf)
    }
}

pub struct FileReaderLease<T: FileReader> {
    f: T,
    pos: Option<usize>,
}

impl<T: FileReader> FileReaderLease<T> {
    pub fn new(f: T) -> Self {
        FileReaderLease { f, pos: None }
    }

    pub async fn read(&mut self, pos: usize, amount: usize) -> Result<Vec<u8>> {
        Ok(self.f.read(amount).await?)
    }

    pub async fn read_u8(&mut self, pos: usize) -> Result<u8> {
        let res = self.f.read(1).await?;
        Ok(res[0])
    }

    pub async fn read_u16(&mut self, pos: usize) -> Result<u16> {
        let res = dbg!(self.f.read(2).await?);
        Ok(LittleEndian::read_u16(&res))
    }

    pub async fn read_u32(&mut self, pos: usize) -> Result<u32> {
        let res = self.f.read(4).await?;
        Ok(LittleEndian::read_u32(&res))
    }
}
