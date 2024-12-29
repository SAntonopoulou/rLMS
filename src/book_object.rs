use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Subject {
    pub name: String,
}

// Structs for additional fields
#[derive(Debug, Deserialize, Clone)]
pub struct Publisher {
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Author {
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Cover {
    pub small: Option<String>,
    pub medium: Option<String>,
    pub large: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkLink {
    pub key: String,
}


/* NOTE TO ADD: [HIGH PRIORITY]
 * We need to adjust the constructor method of the book
 * to include the book_id from the database after it has been
 * created.
 */

/* NOTE TO ADD: [ TOP PRIORITY ]
 * I need to add to this ISBN storage as well
 * as the book_id itself so that I can use these
 * within the program.
 */
#[derive(Debug, Deserialize, Default)]
pub struct Book {
    pub book_id: u32,
    pub isbn: String,
    pub title: String,
    pub authors: Vec<Author>,
    pub publish_date: String,
    pub number_of_pages: Option<u32>,
    pub cover: Option<Cover>,
    pub works: Option<Vec<WorkLink>>,
    pub subjects: Option<Vec<Subject>>,
    pub publishers: Option<Vec<Publisher>>,
}


impl Book {
    pub fn print_book_info(&self) {
        println!("Book ID: {}", self.book_id);
        println!("ISBN: {}", self.isbn);
        println!("Title: {}", self.title);

        // Handle authors
        if !self.authors.is_empty() {
            println!(
                "Author(s): {}",
                self.authors
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            );
        } else {
            println!("Author(s): Not available");
        }

        println!("Publish Date: {}", self.publish_date);

        // Handle number of pages
        if let Some(pages) = self.number_of_pages {
            println!("Number of Pages: {}", pages);
        } else {
            println!("Number of Pages: Not available");
        }

        // Handle cover images
        if let Some(cover) = &self.cover {
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
        } else {
            println!("Cover URLs: Not available");
        }

        // Handle subjects
        if let Some(subjects) = &self.subjects {
            let subject_names: Vec<&str> = subjects.iter().map(|s| s.name.as_str()).collect();
            println!("Subjects: {}", subject_names.join(", "));
        } else {
            println!("Subjects: Not available");
        }

        // Handle publishers
        if let Some(publishers) = &self.publishers {
            let publisher_names: Vec<&str> = publishers.iter().map(|p| p.name.as_str()).collect();
            println!("Publishers: {}", publisher_names.join(", "));
        } else {
            println!("Publishers: Not available");
        }
    }

    pub fn get_id(&self) -> u32 { self.book_id.clone() }
    pub fn get_isbn(&self) -> String { self.isbn.clone() }
    pub fn get_title(&self) -> String { self.title.clone() }
    pub fn get_authors(&self) -> Vec<Author> { self.authors.clone() }
    pub fn get_pub_date(&self) -> String { self.publish_date.clone() }
    pub fn get_number_of_pages(&self) -> Option<u32> { self.number_of_pages }
    pub fn get_works(&self) -> Option<Vec<WorkLink>> { self.works.clone() }
    pub fn get_subjects(&self) -> Option<Vec<Subject>> { self.subjects.clone() }
    pub fn get_publishers(&self) -> Option<Vec<Publisher>> { self.publishers.clone() }
    pub fn has_cover(&self) -> bool { self.cover.is_some() }
    pub fn get_covers(&self) -> Option<Cover> { self.cover.clone() }
    fn get_small_cover(&self) -> Option<&String> {
        self.cover.as_ref()?.small.as_ref()
    }

    fn get_medium_cover(&self) -> Option<&String> {
        self.cover.as_ref()?.medium.as_ref()
    }

    fn get_large_cover(&self) -> Option<&String> {
        self.cover.as_ref()?.large.as_ref()
    }
    pub fn get_cover_by_size(&self, size: &str) -> Option<&String> {
        match size.to_lowercase().as_str() {
            "small" => self.get_small_cover(),
            "medium" => self.get_medium_cover(),
            "large" => self.get_large_cover(),
            _ => None, // Invalid size
        }
    }
}