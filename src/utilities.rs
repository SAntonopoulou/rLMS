use crate::{user_processing, User};
use crate::book_processing;
use crossterm::terminal::ClearType;
use crossterm::terminal::Clear;
use std::{io, thread};
use std::time::Duration;
use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
use crossterm::execute;
use rand::Rng;
use rpassword::read_password;
use validator::ValidateEmail;
use std::env;
use std::path::PathBuf;
use crate::book_object::Book;
use rusqlite::Connection;
use rusqlite::params;
use std::error::Error;
use anyhow::{Context};

pub fn get_email_from_user(db_name: &str) -> anyhow::Result<String, String> {
    loop {
        println!("Enter email: ");

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input. Please try again.");
            continue;
        }

        let email = input.trim().to_lowercase();

        // Validate email format
        if !email.validate_email() {
            println!("Invalid email address. Please try again.");
            continue;
        }

        // Check database for email existence
        match Connection::open(db_name) {
            Ok(connection) => match email_exists(&connection, &email) {
                Ok(true) => {
                    println!("Email '{}' already exists in the database.", email);
                    println!("Please enter a different email.");
                    continue;
                }
                Ok(false) => {
                    return Ok(email.to_string());
                }
                Err(e) => {
                    println!("Failed to check email existence: {}", e);
                    continue;
                }
            },
            Err(e) => {
                eprintln!("Failed to open database: {}", e);
                return Err(format!("Database connection failed: {}", e));
            }
        }
    }
}

pub fn email_exists(conn: &Connection, email: &str) -> anyhow::Result<bool> {
    let mut stmt = conn.prepare("SELECT EXISTS(SELECT 1 FROM users WHERE email = ?1);")?;
    let exists: bool = stmt.query_row([email], |row| row.get(0))?;
    Ok(exists)
}

pub fn generate_salt(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let allowed_chars: Vec<char> = (b'!'..=b'~') // Printable ASCII range
        .filter(|c| {
            // Exclude problematic characters
            !matches!(
                *c as char,
                '\n' | '\r' | '\t' | '\x0B' | '\x0C' // Control characters
            )
        })
        .map(|c| c as char)
        .collect();

    (0..length)
        .map(|_| allowed_chars[rng.gen_range(0..allowed_chars.len())])
        .collect()
}
pub fn get_password_from_user() -> String {
    loop {
        println!("Enter a strong password: ");
        let password = match read_password() {
            Ok(password) => password.trim().to_string(),
            Err(_) => {
                println!("failed to read password. Please try again.");
                continue;
            }
        };

        if !is_safe_password(&password) {
            println!("Password does not meet safety criteria. Please try again.");
            println!("Must include: number, uppercase, lowercase, and special character.");
            continue;
        }

        println!("Re-enter your password to confirm: ");
        let confirm_password = match read_password() {
            Ok(confirm_password) => confirm_password.trim().to_string(),
            Err(_) => {
                println!("Failed to read password confirmation. Please try again.");
                continue;
            }
        };

        if password != confirm_password {
            println!("Passwords do not match. Please try again.");
            continue;
        } else {
            return password.to_string()
        }
    }
}

pub fn get_name_from_user(name_type: &str) -> String {
    loop {
        println!("Enter your {}:", name_type);

        // Step 1: Read user input
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input. Please try again.");
            continue;
        }

        // Step 2: Trim and validate the name
        let name = input.trim();
        if is_valid_name(name) {
            return name.to_string();
        } else {
            println!("Invalid name. Please enter a valid name (alphabetic characters and spaces only).");
        }
    }
}

pub fn print_login_menu(){
    println!("Please choose from the following options:");
    println!("\t1. Login");
    println!("\t2. Register");
    println!("\t3. Exit");
}

pub fn get_yes_or_no() -> bool {
    let mut choice = String::new();
    loop {
        if io::stdin().read_line(&mut choice).is_err() {
            println!("Failed to read input. Please try again.");
            continue;
        }

        let trimmed_lower = choice.trim().to_lowercase();
        if trimmed_lower != "y" && trimmed_lower != "yes" && trimmed_lower != "n" && trimmed_lower != "no" {
            println!("Invalid choice. Please try again.");
        } else {
            match trimmed_lower.as_str() {
                "y" | "yes" => {
                    return true;
                },
                "n" | "no" => {
                    return false;
                },
                _ => {
                    println!("Invalid choice. Please try again.");
                    continue;
                }
            }
        }
    }
}

pub fn get_menu_choice(menu_name: &str) -> usize {
    let mut loop_count: u32 = 0;
    loop {
        loop_count += 1;
        match menu_name {
            "login" => print_login_menu(),
            "user"  => {
                if loop_count > 1 { print_user_menu(false); }
                else { print_user_menu(true) }
            },
            &_ => println!("Invalid menu provided."),
        }
        let mut input = String::new(); // Clear input each iteration.

        println!("Enter your choice for {}:", menu_name);
        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input. Please try again.");
            continue;
        }

        // Attempt to parse the input
        match input.trim().parse::<usize>() {
            Ok(output) => {
                if is_valid_menu_choice(output, menu_name) {
                    return output
                } else {
                    println!("Invalid menu option. Please try again.");
                }
            }
            Err(err) => {
                match err.kind() {
                    std::num::IntErrorKind::PosOverflow => {
                        println!("The number you entered is too large. Please enter a smaller number.");
                    }
                    _ => {
                        println!("Invalid input. Please enter a valid number.");
                    }
                }
            }
        }
    }
}

fn is_valid_menu_choice(choice: usize, menu_name: &str) -> bool {
    match menu_name {
        "login" => {
            if choice >= 1 && choice <= 3 { return true; }
        },
        "user" => {
            if choice >= 0 && choice <= 4 { return true;}
        },
        &_ => {
            println!("Invalid menu option.");
        }
    }

    false
}

pub fn is_valid_name(name: &str) -> bool {
    // Step 3: Ensure the name is non-empty and contains only valid characters
    !name.is_empty() && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace() || c == '-')
}

pub fn is_safe_password(password: &str) -> bool {
    let has_min_length = password.len() >= 8;
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    has_min_length && has_uppercase && has_lowercase && has_digit && has_special
}

pub fn hash_password(password: &str, salt: &str) -> anyhow::Result<String, BcryptError> {
    let salted_password = format!("{}{}", password, salt);
    hash(&salted_password, DEFAULT_COST)
}

pub fn verify_hash(password: &str, hashed: &str) -> anyhow::Result<bool, BcryptError> {
    verify(password, hashed)
}

pub fn pause(seconds: u64) {
    thread::sleep(Duration::from_secs(seconds));
}

pub fn clear_screen() { execute!(io::stdout(), Clear(ClearType::All)); }

pub fn default_config_path() -> PathBuf {
    PathBuf::from("config.json")
}
pub fn get_config_path() -> PathBuf {
    env::var("CONFIG_PATH").map(PathBuf::from).unwrap_or_else(|_| default_config_path())
}

pub fn print_admin_menu_header(){
    println!("======================");
    println!("== rLMS Admin Panel ==");
    println!("======================");
}

pub fn print_user_menu_header() {
    let welcome_string: &str = &("== Welcome back ==");
    let header_footer_length = welcome_string.len();
    let header_footer = "=".repeat(header_footer_length);
    let menu = format!("{}\n{}\n{}", header_footer, welcome_string, header_footer);
    println!("{}", menu);
}

pub fn print_admin_menu(header: bool){
    if header {
        print_admin_menu_header();
    }

    println!(
        "Choose from the options below:\n\
        \t1. List Users\n\
        \t2. Add User\n\
        \t3. Remove User\n
        \t0. Logout\n"
    );
}
pub fn print_user_menu(header: bool) {
    if header {
        print_user_menu_header();
    }
    println!(
        "Choose from the options below:\n\
        \t1. Search Your Books\n\
        \t2. Add Book\n\
        \t3. Delete Book\n\
        \t4. Modify Personal Information\n\
        \t0. Logout\n"
    );
}

pub async fn process_user_menu_choice(choice: usize, user: &User, database_name: &str) -> bool {
    match choice {
        1 => {
            println!("You chose to Search Your Books.");
            // Implement search functionality here
            // For example:
            // search_books().await;
            pause(2);
            true // Continue the loop
        },
        2 => {
            if book_processing::add_new_book_to_collection(database_name, &user).await {
                println!("Book added successfully.");
            } else {
                println!("Failed to add the book.");
            }
            pause(2);
            true // Continue the loop
        },
        3 => {
            println!("You chose to Delete a Book.");
            // Implement delete functionality here
            // For example:
            // delete_book().await;
            if book_processing::delete_book_from_collection(database_name, &user) {
                println!("Book deleted successfully.");
            } else {
                println!("Failed to delete the book.");
            }
            pause(2);
            true // Continue the loop
        },
        4 => {
            println!("You chose to Modify Personal Information.");
            // Implement modification functionality here
            // For example:
            // modify_user_info().await;
            if user_processing::change_personal_information(database_name, &user) {
                println!("Personal information changed successfully.");
            } else {
                println!("Failed to modify personal information.");
            }
            pause(2);
            true // Continue the loop
        },
        0 => {
            println!("Logging out...");
            pause(1);
            false // Exit the loop
        },
        _ => {
            println!("Invalid choice. Please try again.");
            pause(2);
            true // Continue the loop
        },
    }
}
