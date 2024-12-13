mod initialisation;
mod utilities;
mod configuration;

use std::io::Write;
use anyhow::Result;
use validator::ValidateEmail;
use rand::Rng;
use std::error::Error;
use configuration::{Config, setup_config_database_file};

fn main() -> Result<()> {
    utilities::clear_screen();
    std::io::stdout().flush()?;

    let config_path = utilities::get_config_path();

    if let Some(config) = Config::load(config_path.to_str().unwrap()) {
        println!("Configuration file found");
        println!("Loaded configuration: {:?}", config);
    } else {
        println!("Configuration file not found. Running initialisation...");
        utilities::pause(2);
        let mut config = Config::default();
        initialisation::check_initial(&mut config, config_path.to_str().unwrap());
        config.save(config_path.to_str().unwrap())?;
    }
    //
    Ok(())
}