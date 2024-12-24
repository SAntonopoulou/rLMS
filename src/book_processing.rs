use crate::book_object::{Book};

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