use validator::ValidateEmail;
use std::io;
use rpassword::read_password;
use rusqlite::Connection;
use rusqlite::params;
use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
use crate::utilities;

pub fn login_user(database_name: &str) -> bool{
    let email: String = get_user_email();
    let password: String = get_user_password();

    let mut user_id: i32;
    match get_user_id_by_email(database_name, &email) {
        Ok(id) => user_id = id,
        Err(e) => {
            println!("Invalid credentials. Please try again");
            return false;
        }
    }

    let mut user_salt:String;
    match get_user_salt_by_id(database_name, &user_id) {
        Ok(s) => user_salt = s.trim().to_string(),
        Err(e) => {
            println!("Invalid credentials. Please try again");
            return false;
        }
    }

    let mut user_password: String;
    match get_user_password_by_id(database_name, &user_id) {
        Ok(p) => user_password = p,
        Err(e) => {
            println!("Invalid credentials. Please try again");
            return false;
        }
    }


    let salted_password = format!("{}{}", password, user_salt);
    if bcrypt::verify(&salted_password, &user_password).unwrap_or(false) {
        true
    } else {
        println!("Invalid credentials. Please try again.");
        false
    }
}

pub fn get_user_id_by_email(database_name: &str, email: &str) -> Result<i32, rusqlite::Error> {
    let connection = Connection::open(database_name)?;
    let query = "SELECT user_id FROM users WHERE email = ?1";
    connection.query_row(query, params![email], |row| row.get(0))
}

pub fn get_user_salt_by_id(database_name: &str, user_id: &i32) -> Result<String, rusqlite::Error> {
    let connection = Connection::open(database_name)?;
    let query = "SELECT salt FROM salts WHERE user_id = ?1";
    connection.query_row(query, params![user_id], |row| row.get(0))
}

pub fn get_user_password_by_id(database_name: &str, user_id: &i32) -> Result<String, rusqlite::Error> {
    let connection = Connection::open(database_name)?;
    let query = "SELECT password FROM passwords WHERE user_id = ?1";
    connection.query_row(query, params![user_id], |row| row.get(0))
}

pub fn register_user(){

}

fn get_user_password() -> String {
    loop {
        println!("Enter your password: ");
        let password = match read_password() {
            Ok(password) => {
                password.trim().to_string();
                return password;},
            Err(_) => {
                println!("failed to read password. Please try again.");
                continue;
            }
        };
    }
}
fn get_user_email() -> String {
    loop {
        println!("Enter user email: ");

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input. Please try again.");
            continue;
        }

        let email:String = std::string::String::from(input.trim());

        // Validate email format
        if !email.validate_email()  {
            println!("Invalid email address. Please try again.");
            continue;
        }

        return email;
    }
}
