use rustcraft::buffer::{BUFFER_SIZE, BufferReader};
use rustcraft::connection::{Connection, ConnectionState};

use rustcraft::packets::try_next_packet;
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:1337";
    let listener = TcpListener::bind(&addr).await?;

    println!("Listening on: {}", addr);

    loop {
        let (mut socket, peer) = listener.accept().await?;

        println!("\n\nAccepted from {peer}\n----------------------");

        tokio::spawn(async move {
            let mut connection = Connection::new(
                peer.ip().to_string(),
                peer.port(),
                socket,
                ConnectionState::Handshake,
            );
            let mut buffer_reader = BufferReader::new();
            loop {
                let mut temp = [0; BUFFER_SIZE];
                let n = match connection.socket.read(&mut temp).await {
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("read error from {peer}: {e}\n----------------------");
                        return;
                    }
                };

                if let Err(e) = buffer_reader.append(temp, n) {
                    eprintln!("buffer overflow from {peer}");
                    return;
                }

                loop {
                    match try_next_packet(&mut buffer_reader) {
                        Ok(Some(mut packet)) => {
                            connection.process(&mut packet).await;
                        }
                        Ok(None) => break, // need more data
                        Err(e) => {
                            eprintln!("protocol error from {peer}: {e}");
                            eprintln!("{:?}", buffer_reader.unread());
                            return;
                        }
                    }
                }

                if n == 0 {
                    println!("closed by {peer}");
                    return;
                }
            }
        });
    }
}
