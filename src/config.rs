use crate::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

impl Config {
    /// Attempts to read in a `config.toml` file from the root directory of your
    pub async fn load() -> Result<Self> {
        let exists = std::fs::metadata("config.toml").is_ok();

        if !exists {
            //tracing::info!("No configuration file found. Creating one now at `config.toml`.");
            //tracing::info!("You may need to change the default values in order to run your .");

            let config = &toml::to_string(&Config::default())?;
            std::fs::write("config.toml", config)?;
        }

        let path = std::fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&path)?;

        Ok(config)
    }

    pub fn addr(&self) -> SocketAddr {
        SocketAddr::new(
            self.host.parse().expect("Failed to parse hostname"),
            self.port,
        )
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: "127.0.0.1".to_string(),
            port: 7000,
        }
    }
}
