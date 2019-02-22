use crate::error::{Error, Result};

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
