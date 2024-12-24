mod initialisation;
mod utilities;
mod configuration;
mod user_management;
mod user_object;
mod book_processing;
mod book_object;

use std::io::Write;
use anyhow::Result;
use configuration::Config;
use crate::utilities::{clear_screen, get_menu_choice, pause, process_user_menu_choice};
use crate::user_object::User;
use crate::book_processing::get_book_info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    clear_screen();
    std::io::stdout().flush()?;

    // Load configuration
    let config_path = utilities::get_config_path();
    let mut config: Config = Config::default();
    if let Some(loaded_config) = Config::load(config_path.to_str().unwrap()) {
        println!("Configuration file found");
        println!("Loaded configuration: {:?}", loaded_config);
        config = loaded_config;
    } else {
        println!("Configuration file not found. Running initialisation...");
        pause(2);
        initialisation::check_initial(&mut config, config_path.to_str().unwrap());
        config.save(config_path.to_str().unwrap())?;
    }

    /*
    // testing the book functions and object
    let isbn = "9781985086593";
    println!("Fetching information for ISBN: {}", isbn);
    let book = get_book_info(isbn).await?;
    println!("Book information retrieved successfully.\n");
    book.print_book_info();
    pause(10);
     */

    let mut logged_in: bool = false;
    clear_screen();
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
                    let (user_check, is_valid) = user_management::login_user(
                        config.database_file.as_deref().expect("Failed to read configuration file.")
                    );
                    if is_valid {
                        user = user_check;
                        logged_in = true;
                        println!("Login successful!");
                        break;
                    } else {
                        count += 1;
                        println!("Invalid credentials. Attempt {}/3.", count);
                    }
                }
            },
            2 => {
                user_management::register_user(
                    config.database_file.as_deref().expect("Failed to read configuration file.")
                );
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

    let mut run_program: bool = true;
    while run_program {
        clear_screen();
        if user.get_is_admin() {
            utilities::print_admin_menu(true);
            // Implement admin menu functionality here
            // For example:
            // run_program = process_admin_menu_choice(get_menu_choice("admin")).await;
        } else {
            utilities::print_user_menu(true);
            let choice = get_menu_choice("user");
            run_program = process_user_menu_choice(choice).await;
        }
    }
    println!("Exiting program...");
    Ok(())
}
