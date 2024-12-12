use std::path::Path;
use rusqlite::{params, Connection, DatabaseName};
use bcrypt::{hash, verify, BcryptError, BcryptResult, DEFAULT_COST};
use std::io;
use std::io::Write;
use anyhow::Result;
use std::fs;
use std::thread;
use std::time::Duration;
use crossterm::{execute, terminal::{Clear, ClearType}};
use validator::ValidateEmail;
use rpassword::read_password;
use rand::Rng;
use std::error::Error;

fn main() -> Result<()> {
    execute!(std::io::stdout(), Clear(ClearType::All))?;
    std::io::stdout().flush()?;
    let db_path = "db.sqlite";
    check_initial("./");
    /*
    if !Path::new(db_path).exists() {
        eprintln!("Database file '{}' does not exist.", db_path);
        eprintln!("Exiting program.");
        return;
    }

    let connection = match Connection::open("db.sqlite") {
        Ok(connection) => {
            println!("Database connection successfully established.");
            connection
        }
        Err(e) => {
            eprintln!("Failed to establish database connection: {}", e);
            eprintln!("Exiting program.");
            return;
        }
    };
     */
    Ok(())
}

fn get_email_from_user(db_name: &str) -> Result<String, String> {
    loop {
        println!("Enter administrator email: ");

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input. Please try again.");
            continue;
        }

        let email = input.trim();

        // Validate email format
        if !email.validate_email() {
            println!("Invalid email address. Please try again.");
            continue;
        }

        // Check database for email existence
        match Connection::open(db_name) {
            Ok(connection) => match email_exists(&connection, email) {
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


fn email_exists(conn: &Connection, email: &str) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT EXISTS(SELECT 1 FROM users WHERE email = ?1);")?;
    let exists: bool = stmt.query_row([email], |row| row.get(0))?;
    Ok(exists)
}
fn check_initial(directory: &str) -> bool {
    let mut return_value: bool = true;
    println!("Checking if initial setup...");
    pause(2);
    /*
    match check_sqlite_files(directory) {
        Some(files) => {
            println!("This does not seem to be your initial setup.");
            for file in files {
                println!("Using database: {}", file.display());
            }
        }
        None => {
            println!("No SQLite files found.");
            println!("Beginning initial setup...");
            //run_initial_setup();
            println!("Your database will be {}.sqlite.", get_database_name());
        }
    }
     */
    let mut database_name = get_database_name();
    database_name.push_str(".sqlite");
    pause(1);
    println!("Your database file will be:\n\t{}", database_name);

    if !create_initial_tables(&database_name) {
        println!("Failed to initalize tables");
        println!("Program terminating");
        return_value = false;
        return return_value;
    } else {
        println!("Database tables created successfully!");
    }

    match create_initial_administrator(&database_name) {
        Ok(_) => {
            println!("Initial administrator account created successfully!");
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
fn create_initial_administrator(db_name: &str) -> Result<(), Box<dyn Error>> {
    let connection = Connection::open(db_name)?;
    let admin_email = match get_email_from_user(db_name) {
        Ok(email) => email,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                e, // Directly pass the String
            )));
        }
    };
            let admin_firstname = get_name_from_user("firstname");
            let admin_lastname = get_name_from_user("lastname");
            let admin_password = get_password_from_user();
            let admin_salt = generate_salt(25);
            let admin_hashed = hash_password(&*(admin_password + &*admin_salt)).unwrap();

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

fn generate_salt(length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| rng.gen_range(b'A'..=b'z') as char)
        .collect()
}
fn get_password_from_user() -> String {
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
            println!("Password does not meet safety critera. Please try again.");
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

fn get_name_from_user(name_type: &str) -> String {
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

fn is_valid_name(name: &str) -> bool {
    // Step 3: Ensure the name is non-empty and contains only valid characters
    !name.is_empty() && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace() || c == '-')
}

fn is_safe_password(password: &str) -> bool {
    let has_min_length = password.len() >= 8;
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    has_min_length && has_uppercase && has_lowercase && has_digit && has_special
}
fn create_initial_tables(db_name: &str) -> bool {
    println!("Creating initial tables...");
    pause(2);
    match Connection::open(db_name) {
        Ok(connection) => {
            println!("Database connection successfully established.");

            println!("Creating users table...");
            pause(1);
            if !create_user_table(&connection) {
                println!("Could not create users table.");
                return false;
            };
            println!("Successfully created users table.");

            println!("Creating passwords table...");
            pause(1);
            if !create_password_table(&connection) {
                println!("Could not create password table.");
                return false;
            };
            println!("Successfully created passwords table.");

            println!("Creating salts table...");
            pause(1);
            if !create_salt_table(&connection) {
                println!("Could not create salts table.");
                return false;
            };
            println!("Successfully created salts table.");

            println!("Creating admins table...");
            pause(1);
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
    pause(1);
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
fn check_sqlite_files(directory: &str) -> Option<Vec<std::path::PathBuf>> {
    let files = fs::read_dir(directory).ok()?;

    let sqlite_files: Vec<std::path::PathBuf> = files
        .filter_map(|file| file.ok())
        .map(|file| file.path())
        .filter(|path| path.is_file()
            && path.extension().map_or(false, |ext| ext == "sqlite"))
        .collect();

    if sqlite_files.is_empty() {
        None
    } else {
        Some(sqlite_files)
    }

}

fn hash_password(password: &str) -> Result<String, BcryptError> {
    hash(password, DEFAULT_COST)
}

fn verify_hash(password: &str, hashed: &str) -> Result<bool, BcryptError> {
    verify(password, hashed)
}

fn pause(seconds: u64) {
    thread::sleep(Duration::from_secs(seconds as u64));
}