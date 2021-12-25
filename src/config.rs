/**
 * Configuration Management
 * 
 * TODO:
 *  - Make this a singleton
 */
use serde::{Deserialize, Serialize};
use serde_yaml::{self};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub cryptowatch_api_url: String,
    pub discord_webhook_url: String,
    pub db_price_history: String,
}

pub fn load(file_path: &str) -> Config {
    let f = std::fs::File::open(file_path).expect("Could not open config file.");
    let config: Config = serde_yaml::from_reader(f).expect("Could not read config values.");
    return config
}