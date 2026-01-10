pub struct Packet {
    payload: Vec<u8>,
}

impl Packet {
    pub fn new(payload: Vec<u8>) -> Self {
        Self { payload }
    }

    pub fn create(id: u8, mut data: Vec<u8>) -> Self {
        data.insert(0, id);
        Self { payload: data }
    }

    pub fn len(&self) -> usize {
        self.payload.len()
    }

    pub fn id(&self) -> u8 {
        self.payload[0]
    }

    pub fn data(&self) -> &[u8] {
        &self.payload[1..]
    }

    pub fn data_owned(&self) -> Vec<u8> {
        let mut data = self.payload.clone();
        data.split_off(1)
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}
