#[derive(Default)]
pub(crate) struct User {
    user_id: i32,
    email: String,
    firstname: String,
    lastname: String,
    is_admin: bool,
}

impl User {
    pub fn new(user_id: i32, email: String, firstname: String, lastname: String, is_admin_int: i32) -> Self {
        let is_admin = match is_admin_int {
            1 => true,
            2 => false,
            _ => panic!("Invalid value for is admin: {}. Exiting program.", is_admin_int),
        };
        User { user_id, email, firstname, lastname, is_admin }
    }

    pub fn set_user_id(&mut self, user_id: i32) { self.user_id = user_id; }
    pub fn set_email(&mut self, email: &str) { self.email = email.to_string();}
    pub fn set_firstname(&mut self, firstname: &str) { self.firstname = firstname.to_string();}
    pub fn set_lastname(&mut self, lastname: &str) { self.lastname = lastname.to_string();}
    pub fn set_is_admin(&mut self, is_admin: bool) {
        self.is_admin = is_admin;
    }

    pub fn get_user_id(&self) -> i32 { self.user_id }
    pub fn get_email(&self) -> &String { &self.email }
    pub fn get_firstname(&self) -> &String { &self.firstname }
    pub fn get_lastname(&self) -> &String { &self.lastname }
    pub fn get_is_admin(&self) -> bool { self.is_admin }

    pub fn pretty_print(&self) {
        println!("User ID: {}", self.user_id);
        println!("Email: {}", self.email);
        println!("First Name: {}", self.firstname);
        println!("Last Name: {}", self.lastname);
        println!("IsAdmin: {}", self.is_admin);
    }

}