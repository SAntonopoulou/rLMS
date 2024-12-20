use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct Book {
    pub title: String,
    pub authors: Vec<Author>,
    pub publish_date: String,
    pub number_of_pages: Option<u32>,
    pub cover: Option<Cover>,
    pub works: Option<Vec<WorkLink>>,
    pub excerpts: Option<Vec<Excerpt>>
}

#[derive(Debug, Deserialize)]
pub struct Author {
   pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Cover {
    pub small: Option<String>,
    pub medium: Option<String>,
    pub large: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct Excerpt {
    pub comment: Option<String>,
    pub excerpt: String,
}
#[derive(Debug, Deserialize)]
pub struct Work {
    pub excerpts: Option<Vec<Excerpt>>,
}
#[derive(Debug, Deserialize)]
pub struct WorkLink {
    pub key: String,
}
fn is_valid_isbn(isbn: &str) -> bool {
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
 *        a typemismatch of type future. Just a heads-up.
 */
pub async fn get_book_info(isbn: &str) -> Result<Book, Box<dyn std::error::Error>> {
    // Fetch book data
    let url = format!(
        "https://openlibrary.org/api/books?bibkeys=ISBN:{}&format=json&jscmd=data",
        isbn
    );

    let response = reqwest::get(&url).await?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch data: HTTP {}", response.status()).into());
    }

    let text = response.text().await?;

    let json: serde_json::Value = serde_json::from_str(&text)?;

    // The key is "ISBN:{isbn}"
    let key = format!("ISBN:{}", isbn);
    if let Some(book_data) = json.get(&key) {
        // Deserialize initial book data
        let mut book: Book = serde_json::from_value(book_data.clone())?;

        // Extract work key
        if let Some(works) = &book.works {
            if let Some(first_work) = works.first() {
                let work_key = &first_work.key;

                // Fetch work data
                let work_url = format!("https://openlibrary.org{}.json", work_key);
                let work_response = reqwest::get(&work_url).await?;

                if work_response.status().is_success() {
                    let work_text = work_response.text().await?;
                    let work_json: serde_json::Value = serde_json::from_str(&work_text)?;

                    // Deserialize work data
                    let work: Work = serde_json::from_value(work_json.clone())?;

                    // Assign excerpts if available
                    book.excerpts = work.excerpts;
                }
            }
        }

        Ok(book)
    } else {
        Err("Book not found".into())
    }
}

pub fn print_book_info(book: &Book) {
    println!("Title: {}", book.title);
    println!(
        "Author(s): {}",
        book.authors
            .iter()
            .map(|a| a.name.as_str()) // Convert &String to &str
            .collect::<Vec<&str>>()    // Collect into Vec<&str>
            .join(", ")                // Join with ", "
    );
    println!("Publish Date: {}", book.publish_date);
    if let Some(pages) = book.number_of_pages {
        println!("Number of Pages: {}", pages);
    }
    if let Some(cover) = &book.cover {
        println!("Cover URLs:");
        if let Some(small) = &cover.small {
            println!("  Small: {}", small);
        }
        if let Some(medium) = &cover.medium {
            println!("  Medium: {}", medium);
        }
        if let Some(large) = &cover.large {
            println!("  Large: {}", large);
        }
    }
    if let Some(excerpts) = &book.excerpts {
        println!("Excerpts:");
        for (i, excerpt) in excerpts.iter().enumerate() {
            if let Some(comment) = &excerpt.comment {
                println!("  Excerpt {} ({}): {}", i + 1, comment, excerpt.excerpt);
            } else {
                println!("  Excerpt {}: {}", i + 1, excerpt.excerpt);
            }
        }
    }
}