use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use toml;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct API {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DB {
    pub path: String,
    pub pool_min: Option<u8>,
    pub pool_max: Option<u8>,
    pub pragma: Option<String>,
    pub timeout: Option<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub api: API,
    pub db: DB,
}

impl Config {
    pub fn socket_address(&self) -> SocketAddr {
        format!("{}:{}", self.api.address, self.api.port)
            .parse()
            .expect("Invalid address")
    }

    pub fn read_config() -> Config {
        let config_file = "config.toml";
        if !std::path::Path::new(config_file).exists() {
            // Generate and save config file
            let default_config = Config {
                api: API {
                    address: "localhost".to_string(),
                    port: 8080,
                },
                db: DB {
                    path: "database.db".to_string(),
                    pool_min: Some(1),
                    pool_max: Some(10),
                    pragma: Some("journal_mode=WAL".to_string()),
                    timeout: Some(30),
                },
            };
            let toml_string = toml::to_string_pretty(&default_config).unwrap();
            std::fs::write(config_file, toml_string).expect("Failed to write config file");
            eprintln!("Config file not found. Generated a default config file.");
        }

        // Read config from file
        let config: Config = toml::from_str(
            &std::fs::read_to_string(config_file).expect("Failed to read config file")
        ).expect("Failed to parse config file");

        config
    }
}