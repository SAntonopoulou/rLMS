mod initialisation;
mod utilities;
mod configuration;
mod user_management;
mod user_object;
mod book_processing;

use std::io::Write;
use anyhow::Result;
use validator::ValidateEmail;
use rand::Rng;
use std::error::Error;
use configuration::{Config};
use crate::utilities::{clear_screen, get_menu_choice, pause};
use serde::{Deserialize};
use crate::user_object::User;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let book: book_processing::Book = book_processing::get_book_info("9781985086593").await?;
    book_processing::print_book_info(&book);

    let mut logged_in: bool = false;
    utilities::clear_screen();
    let mut user: User = User::default();
    while !logged_in {
        let choice = get_menu_choice("login");
        match choice {
            1 => {
                let mut count: u32 = 1;
                loop {
                    if count >= 3 {
                        println!("Too many login attempts.");
                        break;
                    }
                    let(user_check, is_valid) = user_management::login_user(config.database_file.as_deref().expect("Failed to read configuration file."));
                    if is_valid {
                        user = user_check;
                        logged_in = true;
                        break;
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

    let mut run_program:bool = true;
    while run_program {
        clear_screen();
        if user.get_is_admin(){
            utilities::print_admin_menu_header();
            utilities::print_admin_menu();
        } else {
            utilities::print_user_menu_header(&user.get_firstname());
            utilities::print_user_menu();
        }
        break;
    }
    Ok(())
}

