use crate::{buffer::BufferReader, data_types::{DataError, try_read_varint}};


pub fn try_next_packet(buf: &mut BufferReader) -> Result<Option<Vec<u8>>, DataError> {
    let Some(packet_len_i32) = try_read_varint(buf)? else {
        // not enough bytes for the length prefix
        return Ok(None); 
    };

    if packet_len_i32 < 0 {
        // TODO: should return a proper error
        panic!("Negative packet length");
    }

    let packet_len = packet_len_i32 as usize;

    // payload isn't fully available yet
    if buf.unread().len() < packet_len {
        return Ok(None);
    }

    let packet = buf.unread()[..packet_len].to_vec();
    buf.advance(packet_len);
    Ok(Some(packet))
}
