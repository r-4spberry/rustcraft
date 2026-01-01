use thiserror::Error;
use std::ptr;

pub const BUFFER_SIZE: usize = 2048;
use tokio::io::{self, AsyncReadExt};

#[derive(Error, Debug)]
pub enum BufferError {
    #[error("Source is too big")]
    SourceTooBig
}

pub struct BufferReader {
    buf: [u8; BUFFER_SIZE],
    pos: usize,
    len: usize,
}

impl BufferReader {
    pub fn new() -> Self {
        Self {
            buf: [0u8; BUFFER_SIZE],
            pos: 0,
            len: 0,
        }
    }

    pub fn from_list(buf: [u8; BUFFER_SIZE], len: usize) -> Self {
        Self { buf, len, pos: 0 }
    }

    fn next_byte(&mut self) {
        self.pos += 1;
    }

    pub fn set_filled_len(&mut self, n: usize) {
        self.pos = 0;
        self.len = n;
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        if self.pos >= self.len {
            return None;
        }

        let byte: u8 = self.buf[self.pos];
        self.next_byte();
        Some(byte)
    }

    fn compact(&mut self) {
        if self.pos == 0 {
            return;
        }
        let rem = self.len-self.pos;
        self.buf.copy_within(self.pos..self.len, 0);
        self.pos = 0;
        self.len = rem;
    }

    pub fn append(&mut self, src: [u8; BUFFER_SIZE], n: usize) -> Result<(), BufferError> {
        if self.len + n > BUFFER_SIZE {
            self.compact();
        }

        if self.len + n > BUFFER_SIZE {
            return Err(BufferError::SourceTooBig)
        }

        self.buf[self.len..self.len + n].copy_from_slice(&src[..n]);
        self.len += n;
        Ok(())
    }

    pub fn unread(&self) -> &[u8] {
            &self.buf[self.pos..self.len]
        }

    pub fn advance(&mut self, n: usize) {
        self.pos += n;
    }
}

