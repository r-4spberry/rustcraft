use crate::{
    buffer::BufferReader,
    data_types::{DataError, create_string, create_varint, try_read_varint},
};

pub struct Packet {
    pub id: u8,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn new(id: u8) -> Self {
        Packet {
            id,
            payload: Vec::new(),
        }
    }
    
    pub fn add_payload(&mut self, payload: Vec<u8>) {
        self.payload.extend(payload);
    }
    
    pub fn add_payload_from_bytes(&mut self, bytes: &[u8]) {
        self.payload.extend(bytes);
    }
    
    pub fn sendable(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let id_bytes = create_varint(self.id as i32);
        // id + payload
        let full_payload = [id_bytes, self.payload.clone()].concat();
        let len = create_varint(full_payload.len() as i32);
        buffer.extend(len);
        buffer.extend(full_payload);
        buffer
    }
    
    pub fn sendable_with_id(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let len = create_varint(self.payload.len() as i32);
        buffer.extend(len);
        buffer.push(self.id);
        buffer.extend(self.payload.clone());
        buffer
    }
}

pub fn try_next_packet(buf: &mut BufferReader) -> Result<Option<Packet>, DataError> {
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
    let packet_id = packet[0];
    let packet_payload = packet[1..].to_vec();
    
    let packet = Packet {
        id: packet_id,
        payload: packet_payload,
    };
    
    buf.advance(packet_len);
    Ok(Some(packet))
}
