use std::error::Error;
use std::io;
use std::io::Write;
use rusqlite::{params, Connection, Result};
use crate::book_object::{Book};
use crate::{book_object};
use crate::user_object::User;
use crate::utilities::{clear_screen, get_yes_or_no};
use anyhow::{Context};

pub fn is_valid_isbn(isbn: &str) -> bool {
    let cleaned: String = isbn.chars().filter(|c| c.is_digit(10)).collect();
    match cleaned.len() {
        10 => is_valid_isbn10(&cleaned),
        13 => is_valid_isbn13(&cleaned),
        _ => false,
    }
}

fn is_valid_isbn10(isbn: &str) -> bool {
    if isbn.len() != 10 {
        return false;
    }
    let mut sum = 0;
    for (i, c) in isbn.chars().enumerate() {
        let digit = match c.to_digit(10) {
            Some(d) => d,
            None => return false,
        };
        sum += digit * (10 - i as u32);
    }
    sum % 11 == 0
}

fn is_valid_isbn13(isbn: &str) -> bool {
    if isbn.len() != 13 {
        return false;
    }
    let mut sum = 0;
    for (i, c) in isbn.chars().enumerate() {
        let digit = match c.to_digit(10) {
            Some(d) => d,
            None => return false,
        };
        if i % 2 == 0 {
            sum += digit;
        } else {
            sum += digit * 3;
        }
    }
    sum % 10 == 0
}

/*
 *  Note: As this is an async function you MUST include an .await after
 *        calling this method. Otherwise, you will run into issues with
 *        a type mismatch of type future. Just a heads-up.
 *
 *        This will use the Open Library API so bef sure to check there if
 *        you would like your program to take into account other fields
 *        that they offer.
 */
pub async fn get_book_info(isbn: &str) -> Result<Book, Box<dyn Error>> {
    let trimmed_isbn = isbn.trim(); // Ensure the ISBN is trimmed

    if !is_valid_isbn(trimmed_isbn) {
        return Err("Invalid ISBN".into());
    }

    let url = format!(
        "https://openlibrary.org/api/books?bibkeys=ISBN:{}&format=json&jscmd=data",
        trimmed_isbn
    );

    let response = reqwest::get(&url).await?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch data: HTTP {}", response.status()).into());
    }

    let text = response.text().await?;
    let json: serde_json::Value = serde_json::from_str(&text)?;

    let key = format!("ISBN:{}", trimmed_isbn);
    if let Some(book_data) = json.get(&key) {
        let ol_book: book_object::OpenLibraryBook = serde_json::from_value(book_data.clone())?;
        let book = Book {
            book_id: None,
            isbn: trimmed_isbn.to_string(),
            title: ol_book.title,
            authors: ol_book.authors,
            publish_date: ol_book.publish_date,
            number_of_pages: ol_book.number_of_pages,
            cover: ol_book.cover,
            works: ol_book.works,
            subjects: ol_book.subjects,
            publishers: ol_book.publishers,
        };
        Ok(book)
    } else {
        Err("Book not found".into())
    }
}

fn get_books_by_user(conn: &Connection, user_id: i32) -> Result<Vec<Book>> {
    // Prepare the SQL query
    let mut stmt = conn.prepare(
        "SELECT books.book_id, books.title, books.author, books.isbn
         FROM books
         JOIN libraries ON books.book_id = libraries.book_id
         WHERE libraries.user_id = ?1",
    )?;

    // Execute the query and map the results to the Book struct
    let book_iter = stmt.query_map(params![user_id], |row| {
        Ok(Book {
            book_id: row.get(0)?,
            isbn: row.get(3)?,
            title: row.get(1)?,
            authors: vec![book_object::Author { name: row.get(2)?}],
            publish_date: String::new(),
            number_of_pages: None,
            cover: None,
            works: None,
            subjects: None,
            publishers: None,
        })
    })?;

    // Collect the results into a Vec<Book>
    let mut books = Vec::new();
    for book in book_iter {
        books.push(book?);
    }

    Ok(books)
}

pub(crate) fn delete_book_from_collection(database_name: &str, user: &User) -> bool {
    clear_screen();
    print_delete_book_header();
    let mut see_list: bool = false;
    loop {
        println!("Would you like to see a list of books (if you do not know the book ID)? (y/n):");
        see_list = get_yes_or_no();
        if see_list {
            // Connect to the database
            let connection = match Connection::open(database_name) {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("Failed to connect to the database: {}", e);
                    return false;
                }
            };

            match get_books_by_user(&connection, user.get_user_id()) {
                Ok(books) => {
                    for book in books {
                        println!(
                            "ID: {}, Title: {}, Author: {}, ISBN: {}",
                            book.book_id.map(|id| id.to_string()).unwrap_or_else(|| "Not available".to_string()),
                            book.title,
                            book.authors.get(0).map_or("Unknown Author", |a| a.name.as_str()),
                            book.isbn
                        );
                    }
                }
                Err(e) => println!("Error retrieving books: {}", e),
            }
            break;
        } else {
            break;
        }
    }

    loop {
        println!("Enter the ID of the book you wish to delete: ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut choice = String::new();
        if io::stdin().read_line(&mut choice).is_err() {
            println!("Failed to read input. Please try again.");
            continue;
        }
        let choice = choice.trim();

        match choice.parse::<u32>() {
            Ok(converted_choice) => {
                let connection = match Connection::open(database_name) {
                    Ok(connection) => connection,
                    Err(e) => {
                        eprintln!("Failed to connect to the database: {}", e);
                        return false;
                    }
                };

                if book_exists(&connection, converted_choice).expect("No book with id {converted_choice} exists") {
                    // confirm deletion of book
                    println!("You would like to delete book with the ID {}? (y/n)", converted_choice);
                    if !get_yes_or_no() { continue; }
                    connection.execute(
                        "DELETE FROM libraries WHERE user_id = ?1 AND book_id = ?2",
                        params![user.get_user_id(), converted_choice],
                    ).with_context(|| format!("Failed to delete book with ID {} from user {}", converted_choice, user.get_user_id()));
                    return true;
                } else {
                    println!("There is no book with ID: {}. Please try again.", converted_choice);
                    continue;
                }
            }
            Err(_) => {
                println!("Invalid ID. Please enter a valid book ID.");
                continue
            }
        }
    }
}


fn book_exists(conn: &Connection, book_id: u32) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT EXISTS(SELECT 1 FROM books WHERE book_id = ?)")?;
    let exists: i32 = stmt.query_row(params![book_id], |row| row.get(0))?;

    Ok(exists != 0)
}

fn print_delete_book_header() {
    println!("#################################");
    println!("## Delete Book From Collection ##");
    println!("#################################");
}
fn print_add_book_header() {
    println!("############################");
    println!("## Add Book to Collection ##");
    println!("############################");
}

/* ========= TO DO [ TOP PRIORITY ===========
 * I need to modify this so it checks
 * if a book with the provided ISBN
 * already exists in the library, and
 * if it does I need to merely associate
 * that book with the particular users
 * library.
 *
 * If the book does not already exist
 * by ISBN then the book will be added
 * new to the database.
 */
pub(crate) async fn add_new_book_to_collection(database_name: &str, user: &User) -> bool {
    clear_screen();
    print_add_book_header();
    // get the ISBN from the user
    let mut isbn: String = String::new();
    loop {
        println!("Enter ISBN(10 or 13):");
        isbn.clear();
        if io::stdin().read_line(&mut isbn).is_err() {
            println!("Failed to read input. Please try again.");
            continue;
        }

        let trimmed_isbn = isbn.trim();

        if !is_valid_isbn(trimmed_isbn) {
            println!("Invalid ISBN {}. Please try again.", trimmed_isbn);
        } else {
            // valid ISBN
            break;
        }
    }

    match get_book_info(isbn.trim()).await {
        Ok(book) => {
            upload_book_to_database(book, isbn.trim(), &user, database_name);
        },
        Err(e) => {
            println!("Error fetching book information: {}", e);
        }
    }

    true
}

fn upload_book_to_database(book: Book, isbn: &str, user: &User, database_name: &str) -> anyhow::Result<(), Box<dyn Error>> {
    let connection = Connection::open(database_name)?;
    let primary_author = &book.get_authors()[0].name;
    let title = book.get_title();
    connection.execute(
        "INSERT INTO books (title, author, isbn) VALUES (?1, ?2, ?3)",
        params![title, primary_author, isbn.trim()],
    ).context("Failed to execute INSERT into books table")?;

    let query = "SELECT book_id FROM books WHERE isbn = ?1 AND title = ?2";
    let book_id: i32 = connection.query_row(
        query,
        params![isbn.trim(), title],
        |row| row.get(0)
    ).context("Failed to retrieve book_id from books table")?;

    let user_id = user.get_user_id();

    connection.execute(
        "INSERT INTO libraries (user_id, book_id) VALUES (?1, ?2)",
        params![user_id, book_id],
    ).context("Failed to execute insert into libraries table")?;
    Ok(())
}