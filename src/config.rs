use lazy_static::lazy_static;
use serde_derive::Deserialize;
use std::fs;
use toml;

lazy_static! {
    pub static ref config: Config = Config::new("./config.toml");
}

// this is the basic config, you can later on access the values with config.global.discord_client
#[derive(Deserialize)]
pub struct Config {
    pub global: Global,
}

#[derive(Deserialize, Clone)]
pub struct Global {
    pub discord_client: String,
    pub discord_secret: String,
}

fn open_config(path: &str) -> Config {
    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            panic!("Couldn't open config file due to {e}");
        }
    };

    match toml::from_str(&contents) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("{e}");
            panic!("Unable to load data from {}", path);
        }
    }
}

impl Config {
    pub fn new(path: &str) -> Self {
        let data = open_config(path);
        Config {
            global: data.global,
        }
    }
}
