use rusqlite::params;
use std::error::Error;
use rusqlite::Connection;
use crate::utilities;

pub fn create_initial_tables(db_name: &str) -> bool {
    println!("Creating initial tables...");
    utilities::pause(2);
    match Connection::open(db_name) {
        Ok(connection) => {
            println!("Database connection successfully established.");

            println!("Creating users table...");
            utilities::pause(1);
            if !create_user_table(&connection) {
                println!("Could not create users table.");
                return false;
            };
            println!("Successfully created users table.");

            println!("Creating passwords table...");
            utilities::pause(1);
            if !create_password_table(&connection) {
                println!("Could not create password table.");
                return false;
            };
            println!("Successfully created passwords table.");

            println!("Creating salts table...");
            utilities::pause(1);
            if !create_salt_table(&connection) {
                println!("Could not create salts table.");
                return false;
            };
            println!("Successfully created salts table.");

            println!("Creating admins table...");
            utilities::pause(1);
            if !create_admin_table(&connection) {
                println!("Could not create admins table.");
                return false;
            };
            println!("Successfully created admins table.");

            true
        }
        Err(e) => {
            eprintln!("Failed to establish database connection: {}", e);
            false
        }
    }
}

fn create_admin_table(connection: &Connection) -> bool {
    let result = connection.execute(
        "CREATE TABLE IF NOT EXISTS admins (
            user_id INTEGER UNIQUE,
            FOREIGN KEY (user_id) REFERENCES users(user_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
        );",
        [],
    );

    match result {
        Ok(_) => true,
        Err(e) => false,
    }
}

fn create_salt_table(connection: &Connection) -> bool {
    let result = connection.execute(
        "CREATE TABLE IF NOT EXISTS salts (
            user_id INTEGER UNIQUE,
            salt VARCHAR(200),
            FOREIGN KEY (user_id) REFERENCES users(user_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
        );",
        [],
    );

    match result {
        Ok(_) => true,
        Err(e) => false,
    }
}
fn create_password_table(connection: &Connection) -> bool {
    let result = connection.execute(
        "CREATE TABLE IF NOT EXISTS passwords (
            user_id INTEGER UNIQUE,
            password VARCHAR(200),
            FOREIGN KEY (user_id) REFERENCES users(user_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
        );",
        [],
    );

    match result {
        Ok(_) => true,
        Err(e) => false
    }
}
fn create_user_table(connection: &Connection) -> bool {
    let result = connection.execute(
        "CREATE TABLE IF NOT EXISTS users (
            user_id INTEGER PRIMARY KEY,
            email VARCHAR(200) NOT NULL UNIQUE,
            firstname VARCHAR(255) NOT NULL,
            lastname VARCHAR(255) NOT NULL
        );",
        [],
    );

    match result {
        Ok(_) => true,
        Err(e) => false
    }
}

fn get_database_name() -> String {
    println!("This program uses SQLite to store user data, and we need\nto create a new database");
    utilities::pause(1);
    let mut database = String::new();

    loop {
        println!("Enter a name for your database: ");

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input. Please try again.");
            continue;
        }

        let trimmed = input.trim();
        if is_valid_database_name(trimmed) {
            database = trimmed.to_string();
            break;
        } else {
            println!("Invalid database name. Please try again.");
        }
    }
    database
}

fn is_valid_database_name(db_name: &str) -> bool {
    !db_name.is_empty()
        && db_name.chars().all(|c| c.is_alphanumeric() || c =='_')
        && !db_name.contains('.')
}


pub fn check_initial(directory: &str) -> bool {
    let mut return_value: bool = true;
    println!("Checking if initial setup...");
    utilities::pause(2);

    let mut database_name = get_database_name();
    database_name.push_str(".sqlite");
    utilities::pause(1);
    println!("Your database file will be:\n\t{}", database_name);

    if !create_initial_tables(&database_name) {
        println!("Failed to initalize tables");
        println!("Program terminating");
        return_value = false;
        return return_value;
    } else {
        println!("Database tables created successfully!");
        utilities::pause(1);
    }

    utilities::clear_screen();
    match create_initial_administrator(&database_name) {
        Ok(_) => {
            println!("Initial administrator account created successfully!");
            utilities::pause(1);
        },
        Err(e) => {
            println!("Failed to create initial administrator account: {}", e);
            println!("Program terminating");
            return_value = false;
            return return_value;
        }
    }

    return_value
}

fn create_initial_administrator(db_name: &str) -> anyhow::Result<(), Box<dyn Error>> {
    let connection = Connection::open(db_name)?;
    let admin_email = match utilities::get_email_from_user(db_name) {
        Ok(email) => email,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                e, // Directly pass the String
            )));
        }
    };
    let admin_firstname = utilities::get_name_from_user("firstname");
    let admin_lastname = utilities::get_name_from_user("lastname");
    let admin_password = utilities::get_password_from_user();
    let admin_salt = utilities::generate_salt(25);
    let admin_hashed = utilities::hash_password(&*(admin_password + &*admin_salt)).unwrap();

    // enter admin data into users entity
    connection.execute(
        "INSERT INTO users (email, firstname, lastname) VALUES (?1, ?2, ?3)",
        params![admin_email, admin_firstname, admin_lastname],
    )?;

    // enter admin data into salts entity
    connection.execute(
        "INSERT INTO salts (user_id, salt) VALUES (
                    (SELECT user_id FROM users WHERE email = ?1), ?2)",
        params![admin_email, admin_salt],
    )?;

    // enter admin data into passwords entity
    connection.execute(
        "INSERT INTO passwords (user_id, password) VALUES (
                (SELECT user_id FROM users WHERE email = ?1), ?2)",
        params![admin_email, admin_hashed],
    )?;

    // enter admin data into admins entity
    connection.execute(
        "INSERT INTO admins (user_id) VALUES (
                (SELECT user_id FROM users WHERE email = ?1))",
        params![admin_email],
    )?;

    Ok(())
}