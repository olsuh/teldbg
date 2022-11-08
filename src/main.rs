mod connection;
mod config;
mod telnet;
mod command;
mod mem_dump;

use telnet::telnet_connection_loop;
use config::Config;

use tokio::net::TcpListener;
use tracing;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {

    let config = Config::load().await?;

    let telnet_listener = TcpListener::bind(config.addr()).await?;
    tracing::info!("teldbg listening on {}", config.addr());

    loop {
        tokio::select! {
            Ok((stream, addr)) = telnet_listener.accept() => {
                tokio::spawn(async move {
                    tracing::info!("New connection from {}", addr);
                    if let Err(e) = telnet_connection_loop(stream, addr).await {
                        tracing::error!(%e, "Error handling telnet connection");
                    }
                });
            }
        }
    }
}