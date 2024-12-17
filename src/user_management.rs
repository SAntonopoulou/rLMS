use validator::ValidateEmail;
use std::io;
use rpassword::read_password;
use rusqlite::Connection;
use rusqlite::params;
use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
use crate::utilities;
use crate::user_object;
use std::error::Error;

pub fn login_user(database_name: &str) -> (user_object::User, bool){
    utilities::clear_screen();
    println!("==============================");
    println!("===       User Login       ===");
    println!("==============================");

    let mut user = user_object::User::default();

    let email: String = get_user_email();
    let password: String = get_user_password();

    let mut user_id: i32;
    match get_user_id_by_email(database_name, &email) {
        Ok(id) => user_id = id,
        Err(e) => {
            println!("Invalid credentials. Please try again");
            return (user, false);
        }
    }

    let mut user_salt:String;
    match get_user_salt_by_id(database_name, &user_id) {
        Ok(s) => user_salt = s.trim().to_string(),
        Err(e) => {
            println!("Invalid credentials. Please try again");
            return (user, false);
        }
    }

    let mut user_password: String;
    match get_user_password_by_id(database_name, &user_id) {
        Ok(p) => user_password = p,
        Err(e) => {
            println!("Invalid credentials. Please try again");
            return (user, false);
        }
    }


    let salted_password = format!("{}{}", password, user_salt);
    if bcrypt::verify(&salted_password, &user_password).unwrap_or(false) {
        let mut firstname: String;
        match get_user_firstname_by_id(database_name, &user_id) {
            Ok(s) => firstname = s,
            Err(e) => {
                println!("Invalid credentials. Please try again");
                return (user, false);
            }
        }

        let mut lastname: String;
        match get_user_lastname_by_id(database_name, &user_id) {
            Ok(s) => lastname = s,
            Err(e) => {
                println!("Invalid credentials. Please try again");
                return (user, false);
            }
        }
        let mut is_admin = match get_user_is_admin_by_id(database_name, &user_id){
            Ok(true) => true,
            Ok(false) => false,
            Err(_) => {
                println!("Failed to determine admin status.");
                false
            }
        };

        user.set_user_id(user_id);
        user.set_email(&email);
        user.set_firstname(&firstname);
        user.set_lastname(&lastname);
        user.set_is_admin(is_admin);
        (user, true)
    } else {
        println!("Invalid credentials. Please try again.");
        (user, false)
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

pub fn get_user_firstname_by_id(database_name: &str, user_id: &i32) -> Result<String, rusqlite::Error> {
    let connection = Connection::open(database_name)?;
    let query = "SELECT firstname FROM users WHERE user_id = ?1";
    connection.query_row(query, params![user_id], |row| row.get(0))
}

pub fn get_user_lastname_by_id(database_name: &str, user_id: &i32) -> Result<String, rusqlite::Error> {
    let connection = Connection::open(database_name)?;
    let query = "SELECT lastname FROM users WHERE user_id = ?1";
    connection.query_row(query, params![user_id], |row| row.get(0))
}

pub fn get_user_is_admin_by_id(database_name: &str, user_id: &i32) -> Result<bool, rusqlite::Error> {
    let connection = Connection::open(database_name)?;
    let query = "SELECT COUNT(*) FROM admins WHERE user_id = ?1";
    let count: i32 = connection.query_row(query, params![user_id], |row| row.get(0))?;
    Ok(count > 0)
}
pub fn register_user(database_name: &str){
    utilities::clear_screen();
    println!("=============================");
    println!("=== New User Registration ===");
    println!("=============================");
    let mut email: String;
    let mut firstname: String;
    let mut lastname: String;
    let mut password: String;
    let mut salt: String;
    let mut hashed: String;

    loop {
        email= utilities::get_email_from_user(database_name).expect("REASON");
        firstname = utilities::get_name_from_user("firstname");
        lastname = utilities::get_name_from_user("lastname");
        password = utilities::get_password_from_user();
        salt = utilities::generate_salt(25).trim().to_string();
        hashed = utilities::hash_password(&password, &salt).unwrap();

        if !confirm_user_information(&email, &firstname, &lastname) { continue; }
        else {break; }
    }

    match create_new_user(database_name, &email, &firstname, &lastname, &salt, &hashed) {
        Ok(_) => {
            println!("User {} account created successfully!", email);
            utilities::pause(1);
        },
        Err(e) => {
            println!("Failed to create user {} account: {}",email, e);
            utilities::pause(1);
        }
    }
}

fn create_new_user(database_name: &str, email: &str, firstname: &str, lastname: &str, salt: &str, hashed: &str) -> anyhow::Result<(), Box<dyn Error>> {
    let connection = Connection::open(database_name)?;

    connection.execute(
        "INSERT INTO users (email, firstname, lastname) VALUES (?1, ?2, ?3)",
        params![email, firstname, lastname],
    )?;

    connection.execute(
        "INSERT INTO salts (user_id, salt) VALUES (
                    (SELECT user_id FROM users WHERE email = ?1), ?2)",
        params![email, salt],
    )?;

    connection.execute(
        "INSERT INTO passwords (user_id, password) VALUES (
                (SELECT user_id FROM users WHERE email = ?1), ?2)",
        params![email, hashed],
    )?;

    Ok(())
}
fn confirm_user_information(email: &str, firstname: &str, lastname: &str) -> bool {
    loop {
        println!("Please confirm the following information: ");
        println!("\tEmail: {}", email);
        println!("\tFirst Name: {}", firstname);
        println!("\tLast Name: {}", lastname);
        println!("Are these correct (y/n): ");
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input. Please try again.");
            continue;
        }
        let x = if input.to_lowercase().trim() == "y" || input.to_lowercase().trim() == "yes"
        { return true; }
        else if input.to_lowercase().trim() == "n" || input.to_lowercase().trim() == "no"
        { return false; }
        else
        {
            println!("Invalid input. Please try again.");
            continue
        };
    }
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

        let email:String = std::string::String::from(input.trim()).to_lowercase();

        // Validate email format
        if !email.validate_email()  {
            println!("Invalid email address. Please try again.");
            continue;
        }

        return email;
    }
}
