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

pub fn get_server_info_json() -> Result<String, Box<dyn std::error::Error>> {
    let toml_str = fs::read_to_string("server_info.toml")?;
    let data: ServerInfo = toml::from_str(&toml_str)?;
    let json = serde_json::to_string_pretty(&data)?;
    Ok(json)
}
