use tokio::net::TcpListener;
//use tokio::prelude::*;

use tracing;

mod connection;
mod config;
mod telnet;
pub mod command;
mod mem_dump;
use telnet::telnet_connection_loop;
use config::Config;

//pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
//pub async fn listen(&self, world: World) -> Result<()> {

    // Loads the `config.toml` file in the current directory. We also set the
    // environment variable for our database here, which means we MUST
    // create the state AFTER these functions.
    let config = Config::load().await?;

    // Creates our connection listener
    let telnet_listener = TcpListener::bind(config.addr()).await?;
    tracing::info!("Server listening on {}", config.addr());

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