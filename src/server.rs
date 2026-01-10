use std::collections::HashMap;

use log::{debug, error, info};
use tokio::sync::mpsc;

use crate::messages::{ServerMsg, ServerPacket};

use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct ServerInfo {
    version: Version,
    players: Players,
    description: Description,
    favicon: String,
    enforcesSecureChat: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    name: String,
    protocol: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Players {
    max: u32,
    online: u32,
    #[serde(default)]
    sample: Vec<PlayerSample>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerSample {
    name: String,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Description {
    text: String,
}

pub fn get_server_info_json() -> anyhow::Result<String> {
    let toml_str = fs::read_to_string("server_info.toml")?;
    let data: ServerInfo = toml::from_str(&toml_str)?;
    let json = serde_json::to_string_pretty(&data)?;
    Ok(json)
}

pub async fn run_core(mut core_rx: mpsc::Receiver<ServerMsg>) {
    info!("booting core up!");
    let mut conns: HashMap<usize, mpsc::Sender<ServerPacket>> = HashMap::new();
    while let Some(cmd) = core_rx.recv().await {
        debug!("got command: {cmd:?}");
        match cmd {
            ServerMsg::Register { conn_id, out_tx } => {
                info!("[{conn_id}] Register");
                conns.insert(conn_id, out_tx);
            }
            ServerMsg::Unregister { conn_id } => {
                info!("[{conn_id}] Unregister");
                conns.remove(&conn_id);
            }
            ServerMsg::Ping { conn_id, data } => {
                info!("[{conn_id}] Pong");
                if let Some(out_tx) = conns.get(&conn_id) {
                    if let Err(e) = out_tx.send(ServerPacket::Pong(data)).await {
                        error!("{e}");
                    }
                }
            }
            ServerMsg::Status { conn_id } => {
                info!("[{conn_id}] Status");
                if let Some(out_tx) = conns.get(&conn_id) {
                    match get_server_info_json() {
                        Ok(answer) => {
                            if let Err(e) = out_tx.send(ServerPacket::Status(answer)).await {
                                error!("{e}");
                            }
                        }
                        Err(e) => error!("{e}"),
                    }
                }
            }
            _ => unimplemented!(),
        }
    }
}
