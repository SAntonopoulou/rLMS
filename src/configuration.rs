use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub database_file: Option<String>,
}

impl Config {
    pub fn load(path: &str) -> Option<Self> {
        fs::read_to_string(path).ok().and_then(|data| serde_json::from_str(&data).ok())
    }
    pub(crate) fn save(&self, path: &str) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    fn exists(path: &str) -> bool {
        fs::metadata(path).is_ok()
    }
}

pub fn setup_config_database_file(config: &mut Config, database_file: &str, path: &str) {
    config.database_file = Some(database_file.to_string());
    if let Err(e) = config.save(path) {
        println!("Failed to save configuration: {}", e);
    }

}

