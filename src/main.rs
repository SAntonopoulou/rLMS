mod initialisation;
mod utilities;
mod configuration;
mod user_management;
mod user_object;

use std::io::Write;
use anyhow::Result;
use validator::ValidateEmail;
use rand::Rng;
use std::error::Error;
use configuration::{Config};
use crate::utilities::get_menu_choice;
use serde::Deserialize;

fn main() -> Result<()> {
    utilities::clear_screen();
    std::io::stdout().flush()?;

    let config_path = utilities::get_config_path();
    let mut config: Config = Config::default();
    if let Some(loaded_config) = Config::load(config_path.to_str().unwrap()) {
        println!("Configuration file found");
        println!("Loaded configuration: {:?}", loaded_config);
        config = loaded_config;
    } else {
        println!("Configuration file not found. Running initialisation...");
        utilities::pause(2);
        initialisation::check_initial(&mut config, config_path.to_str().unwrap());
        config.save(config_path.to_str().unwrap())?;
    }

    utilities::clear_screen();
    loop {
        let choice = get_menu_choice("login");
        match choice {
            1 => {
                let mut count: u32 = 1;
                loop {
                    if count >= 3 {
                        println!("Too many login attempts.");
                        break;
                    }

                    let(user, is_valid) = user_management::login_user(config.database_file.as_deref().expect("Failed to read configuration file."));
                    if is_valid {
                        println!("Logged in Successfully.");
                        // DEBUGGING
                        user.pretty_print();
                        break;
                        // END DEBUGGING
                    } else {
                        count+=1;
                    }
                }
            },
            2 => {
                user_management::register_user(config.database_file.as_deref().expect("Failed to read configuration file."));
            },
            3 => {
                println!("Exiting program...");
                std::process::exit(0);
            },
            0_usize | 4_usize.. => {
                println!("Exiting program...");
                std::process::exit(0);
            }
        }
    }
}