#[derive(Debug)]
pub struct UserData {
    server: String,
    username: String,
}

impl UserData {
    pub fn new(server: String, uname: String) -> Self {
        Self {
            server,
            username: uname,
        }
    }

    pub fn server(&self) -> &String {
        &self.server
    }

    pub fn username(&self) -> &String {
        &self.username
    }
}
