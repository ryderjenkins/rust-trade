use serde::Deserialize;
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub max_lifetime: u64,
}

#[derive(Debug, Deserialize)]
pub struct Api {
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: Database,
    pub api: Api,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            .add_source(File::with_name("../config/default"))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .add_source(Environment::with_prefix("APP"))
            .build()?;

        println!("Config content: {:?}", s);
        
        s.try_deserialize()
    }
}