use std::thread::current;

use thiserror::Error;

use crate::buffer::BufferReader;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("VarInt too big")]
    VarIntTooBig,
    #[error("Not enough bytes for VarInt")]
    VarIntNotEnough,
}

pub enum Data {
    VarInt(i32),
    String(String),
    UnsignedShort(u16),
}

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;
pub fn try_read_varint(buf: &mut BufferReader) -> Result<Option<i32>, DataError> {
    let mut value: i32 = 0;
    let mut position: u8 = 0;
    let mut i: usize = 0;
    let bytes = buf.unread();
    loop {
        if i >= bytes.len() {
            //Not enough bytes
            return Ok(None);
        }

        let current_byte = bytes[i];

        value |= ((current_byte & SEGMENT_BITS) as i32) << position;

        if current_byte & CONTINUE_BIT == 0 {
            i += 1;
            buf.advance(i);
            break;
        }

        position += 7;

        if position >= 32 {
            // No need to advance here, breaking
            return Err(DataError::VarIntTooBig);
        }
    }
    Ok(Some(value))
}
