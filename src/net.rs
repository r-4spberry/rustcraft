use tokio::{
    io::AsyncWriteExt,
    net::{
        TcpListener,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::mpsc,
};

use crate::{
    framing::read_packet,
    messages::{ServerMsg, ServerPacket},
    packet::Packet,
    protocol::{self, create_sendable_packet, create_string},
};

use log::{debug, error, info, warn};

pub async fn run_acceptor(
    listener: TcpListener,
    core_tx: mpsc::Sender<ServerMsg>,
) -> anyhow::Result<()> {
    let mut next_id = 1;
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("Accepted connection from {}", addr);
                let conn_id = next_id;
                next_id += 1;

                let core_tx = core_tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = run_connection(stream, core_tx, conn_id).await {
                        error!("conn {conn_id} error: {e:?}");
                    }
                });
            }
            Err(e) => error!("Failed to accept connection: {}", e),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Transfer,
}

async fn run_connection(
    stream: tokio::net::TcpStream,
    core_tx: mpsc::Sender<ServerMsg>,
    conn_id: usize,
) -> anyhow::Result<()> {
    info!("Starting connection {}", conn_id);
    let (read, write) = stream.into_split();
    let (out_tx, out_rx) = mpsc::channel::<ServerPacket>(256);

    core_tx
        .send(ServerMsg::Register {
            conn_id,
            out_tx: out_tx.clone(),
        })
        .await
        .ok();

    tokio::spawn(writer_loop(write, out_rx));

    let mut state = ConnectionState::Handshake;
    reader_loop(read, conn_id, &mut state, out_tx, &core_tx).await?;

    core_tx.send(ServerMsg::Unregister { conn_id }).await.ok();

    Ok(())
}

async fn writer_loop(mut w: OwnedWriteHalf, mut out_rx: mpsc::Receiver<ServerPacket>) {
    while let Some(pkt) = out_rx.recv().await {
        let to_send = match pkt {
            ServerPacket::Status(answer) => {
                let data = create_string(&answer);
                let packet = Packet::create(0x00, data);
                create_sendable_packet(packet)
            }
            ServerPacket::Pong(answer) => {
                let packet = Packet::create(0x01, answer);
                create_sendable_packet(packet)
            }
        };
        if let Err(e) = w.write(&to_send).await {
            error!("Error in writer loop: {e}")
        };
    }
}

async fn reader_loop(
    mut r: OwnedReadHalf,
    conn_id: usize,
    state: &mut ConnectionState,
    out_tx: mpsc::Sender<ServerPacket>,
    core_tx: &mpsc::Sender<ServerMsg>,
) -> anyhow::Result<()> {
    loop {
        info!("Awaiting next frame form {conn_id}");
        let packet = match read_packet(&mut r).await? {
            Some(p) => p,
            None => {
                info!("{conn_id} disconnected");
                return Ok(());
            }
        };

        debug!(
            "got frame from {} [{:?}]: [{:#02x}]{:#02x?}",
            conn_id,
            state,
            packet.id(),
            packet.data()
        );
        match (*state, packet.id()) {
            //handshake
            (ConnectionState::Handshake, 0x00) => {
                let next_state = protocol::parse_handshake_next_state(packet)?;
                *state = match next_state {
                    ConnectionState::Handshake => {
                        anyhow::bail!("handshake can't request another handshake")
                    }
                    ConnectionState::Login => ConnectionState::Login,
                    ConnectionState::Status => ConnectionState::Status,
                    ConnectionState::Transfer => anyhow::bail!("transfer is not supported"),
                };
            }
            //status
            (ConnectionState::Status, 0x00) => {
                core_tx.send(ServerMsg::Status { conn_id }).await?;
            }
            //ping
            (ConnectionState::Status, 0x01) => {
                core_tx
                    .send(ServerMsg::Ping {
                        conn_id,
                        data: packet.data_owned(),
                    })
                    .await?;
            }
            _ => error!("Unknown state/id pair: ({:?}{})", state, packet.id()),
        }
    }
}
