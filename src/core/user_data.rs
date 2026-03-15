#[derive(Debug)]
pub struct UserData {
    server: String,
    username: String,
}

impl UserData {
    pub fn load() -> Self {
        use std::io::{self, Write};

        let mut server = String::new();
        let mut username = String::new();

        print!("Enter server domain: ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut server)
            .expect("Failed to read <server>");

        print!("Enter username: ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut username)
            .expect("Failed to read <server>");

        server = server.trim().to_string();
        username = username.trim().to_string();

        UserData { server, username }
    }

    pub fn server(&self) -> &String {
        &self.server
    }

    pub fn username(&self) -> &String {
        &self.username
    }
}
