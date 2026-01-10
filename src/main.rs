mod server;
mod net;
mod messages;
mod framing;
mod protocol;
mod packet;

use dotenv::dotenv;
use log::{error, info, warn};
use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use crate::net::run_acceptor;
use crate::server::run_core;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    
    if std::env::var_os("RUST_LOG").is_none() {
        panic!("RUST_LOG should be set in env");
    }
    
    env_logger::init();        
    
    info!("Starting server...");
    let addr = env::var("SERVER_ADDR").unwrap_or_else(|_| {
        warn!("SERVER_ADDR environment variable not set, using default address");
        "127.0.0.1:1333".to_string()
    });

    let listener = TcpListener::bind(&addr).await.unwrap();
    info!("Listening on {}", addr);

    let (core_tx, core_rx) = mpsc::channel::<messages::ServerMsg>(1024);
    tokio::spawn(server::run_core(core_rx));
    run_acceptor(listener, core_tx).await?;
    Ok(())
}
