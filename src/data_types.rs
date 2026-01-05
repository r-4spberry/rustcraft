use std::thread::current;

use thiserror::Error;

use crate::buffer::BufferReader;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("VarInt too big")]
    VarIntTooBig,
    #[error("Not enough bytes for this data type")]
    NotEnoughBytes,
}

pub enum Data {
    VarInt(i32),
    String(String),
    UnsignedShort(u16),
}

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;


pub fn create_string(data: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let length = create_varint(data.len() as i32);
    bytes.extend(length);
    bytes.extend(data.as_bytes());
    bytes
}

pub fn create_varint(value: i32) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut value = value as u32;

    loop {
        let mut byte = (value & SEGMENT_BITS as u32) as u8;
        value >>= 7;

        if value != 0 {
            byte |= CONTINUE_BIT;
        }

        bytes.push(byte);

        if value == 0 {
            break;
        }
    }

    bytes
}

pub fn create_long(value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();

    for i in 0..8 {
        bytes.push((value >> (i * 8)) as u8);
    }

    bytes
}

pub fn create_unsigned_short(value: u16) -> Vec<u8> {
    let mut bytes = Vec::new();

    bytes.push((value >> 8) as u8);
    bytes.push(value as u8);

    bytes
}

pub fn read_long(bytes: &mut Vec<u8>) -> Result<u64, DataError> {
    if bytes.len() < 8 {
        return Err(DataError::NotEnoughBytes);
    }
    let mut value = 0u64;
    for i in 0..8 {
        value |= ((bytes[i] as u64) << (i * 8));
    }
    bytes.drain(..8);
    Ok(value)
}

pub fn read_unsigned_short(bytes: &mut Vec<u8>) -> Result<u16, DataError> {
    if bytes.len() < 2 {
        return Err(DataError::NotEnoughBytes);
    }
    let value = ((bytes[0] as u16) << 8) | bytes[1] as u16;
    bytes.drain(..2);
    Ok(value)
}

pub fn read_string(bytes: &mut Vec<u8>) -> Result<String, DataError> {
    let length = read_varint(bytes)? as usize;
    let string = String::from_utf8_lossy(&bytes[..length]).to_string();
    bytes.drain(..length);
    Ok(string)
}

pub fn read_varint(bytes: &mut Vec<u8>) -> Result<i32, DataError> {
    let mut value: i32 = 0;
    let mut position: u8 = 0;
    let mut i: usize = 0;
    loop {
        if i >= bytes.len() {
            // Not enough bytes
            return Err(DataError::NotEnoughBytes);
        }

        let current_byte = bytes[i];
        value |= ((current_byte & SEGMENT_BITS) as i32) << position;

        if current_byte & CONTINUE_BIT == 0 {
            i += 1;
            bytes.drain(..i);
            break;
        }

        position += 7;
        i += 1;
        
        if position >= 32 {
            // No need to drain here, breaking
            return Err(DataError::VarIntTooBig);
        }
    }
    Ok(value)
}

// Technically, this is the only function that *needs* to give out an Option because everything else should be called on a full packet
// this one may be used to get length of a packet - so it can be broken.
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
