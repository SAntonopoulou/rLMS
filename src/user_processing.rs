use crate::user_object::User;
use crate::utilities::{clear_screen, get_yes_or_no};

pub(crate) fn change_personal_information(database_name: &str, user: &User) -> bool {
    clear_screen();
    print_change_personal_information_header();
    loop {
        print_change_info_menu();

        // temporary
        break;
    }
    // temporary
    true
}

fn print_change_info_menu() {
    println!("Which information would you like to change?");
    println!("\t1. First Name");
    println!("\t2. Last Name");
    println!("\t3. Email");
    println!("\t4. Password");
    println!("Enter your choice (1-4)");
}
fn print_change_personal_information_header() {
    println!("#################################");
    println!("## Change Personal Information ##");
    println!("#################################");
}