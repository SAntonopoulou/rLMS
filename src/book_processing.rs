use std::error::Error;
use std::io;
use rusqlite::{params, Connection, Result};
use crate::book_object::{Book};
use crate::{book_processing, utilities};
use crate::user_object::User;
use crate::utilities::clear_screen;
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
pub async fn get_book_info(isbn: &str) -> Result<Book, Box<dyn std::error::Error>> {
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
        let book: Book = serde_json::from_value(book_data.clone())?;
        Ok(book)
    } else {
        Err("Book not found".into())
    }
}

/* ========================== */
/* THIS IS STILL IN PROGRESS! */
/* ========================== */
/* NOTE: I also must add to this the ISBN and the
 *       book_id as those are the fields I wish the
 *       user to be able to use to delete the book.
 *       This will require adjustments to the book
 *       object as well. 
 */
pub(crate) fn delete_book_from_collection(database_name: &str, user: &User) -> bool {
    clear_screen();
    print_delete_book_header();
    let mut see_list: bool = false;
    loop {
        println!("Would you like to see a list of books (if you do not know the ISBN)? (y/n):");
        see_list = utilities::get_yes_or_no();
        if see_list {
            // Connect to the database
            let connection = match Connection::open(database_name) {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("Failed to connect to the database: {}", e);
                    return false;
                }
            };

            let mut query = match connection.prepare("SELECT * FROM books") {
                Ok(query) => query,
                Err(e) => {
                    eprintln!("Failed to prepare SQL statement: {}", e);
                    return false;
                }
            };

            let book_iterator = match query.query_map([], |row| {
                Ok(Book {
                    title: row.get(1)?,
                    authors: vec![crate::book_object::Author { name: row.get(2)?}],
                    publish_date: String::new(),
                    number_of_pages: None,
                    cover: None,
                    works: None,
                    subjects: None,
                    publishers: None,
                })
            }) {
                Ok(iter) => iter,
                Err(e) => {
                    eprintln!("Failed to query books: {}", e);
                    return false;
                }
            };

            for book_result in book_iterator {
                match book_result {
                    Ok(book) => {
                        book.print_book_info();
                        println!("-----------------------------------");
                    }
                    Err(e) => eprintln!("Error retreiving book: {}", e),
                }
            }
            utilities::pause(120);
            /* start deletion process */
            break;
        } else {
            break;
        }
    }
    true
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