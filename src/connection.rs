use crate::data_types::{
    create_long, create_string, create_varint, read_long, read_string, read_unsigned_short, read_varint
};
use crate::packets::Packet;
use crate::server::get_server_info_json;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub struct Connection {
    ip: String,
    port: u16,
    pub socket: tokio::net::TcpStream,
    pub state: ConnectionState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Transfer,
}

impl Connection {
    pub fn new(
        ip: String,
        port: u16,
        socket: tokio::net::TcpStream,
        state: ConnectionState,
    ) -> Self {
        Connection {
            ip,
            port,
            socket,
            state,
        }
    }
}

impl Connection {
    // Send packet back to client
    pub async fn send_packet(&mut self, packet: Packet) {
        let data = packet.sendable();
        let buffer = data.as_slice();
        self.socket.write_all(buffer).await.unwrap();
    }

    pub async fn process(&mut self, packet: &mut Packet) {
        println!("got packet payload len={}", packet.payload.len());
        println!("packet id: {}", packet.id);
        println!("packet payload: {:X?}", packet.payload);
        println!("connection state: {:?}", self.state);
        match self.state {
            ConnectionState::Handshake => {
                let protocol_version = read_varint(&mut packet.payload).unwrap();
                let server_address = read_string(&mut packet.payload).unwrap();
                let server_port = read_unsigned_short(&mut packet.payload).unwrap();
                let next_state = read_varint(&mut packet.payload).unwrap();
                println!("Handshake: ");
                println!("protocol version: {}", protocol_version);
                println!("server address: {}", server_address);
                println!("server port: {}", server_port);
                println!("next state: {}", next_state);
                match next_state {
                    1 => self.state = ConnectionState::Status,
                    2 => self.state = ConnectionState::Login,
                    3 => self.state = ConnectionState::Transfer,
                    _ => {
                        eprintln!("unknown handshake state {}", next_state);
                        return;
                    }
                }
            }
            ConnectionState::Status => {
                match packet.id {
                    //status_request
                    0x00 => {
                        let response_json = get_server_info_json().unwrap();
                        println!("Sending status response");
                        let mut packet = Packet::new(0x00);
                        let s = create_string(&response_json);
                        packet.add_payload(s);
                        self.send_packet(packet).await;
                    }
                    //ping_request
                    0x01 => {
                        let payload = read_long(&mut packet.payload).unwrap();
                        let mut packet = Packet::new(0x01);
                        packet.add_payload(create_long(payload));
                        self.send_packet(packet).await;
                    }
                    _ => {
                        panic!("Unknown protocol in Status")
                    }
                }
            }
            _ => {
                todo!();
            }
        }
    }
}
