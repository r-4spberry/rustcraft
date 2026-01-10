use tokio::sync::mpsc;

#[derive(Debug)]
pub enum ServerMsg {
    Register {
        conn_id: usize,
        out_tx: mpsc::Sender<ServerPacket>,
    },
    Unregister {
        conn_id: usize,
    },
    Ping {
        conn_id: usize,
        data: Vec<u8>,
    },
    Status {
        conn_id: usize
    }
}

pub enum ServerPacket {
    Pong(Vec<u8>),
    Status(String),
}
