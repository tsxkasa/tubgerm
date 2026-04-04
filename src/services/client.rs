use std::fmt::Debug;

use color_eyre::eyre::{Result, eyre};
use submarine::{Client, auth::AuthBuilder};

#[derive(Default, Debug)]
pub struct ClientService {
    client: Option<Client>,
    current_user: Option<String>,
}

impl ClientService {
    pub async fn create_client(&mut self, url: &str, uname: &str, pw: &str) -> Result<()> {
        self.current_user = Some(uname.to_string());
        let auth = AuthBuilder::new(uname, "1.16.1")
            .client_name("org.tubgerm")
            .hashed(pw);
        let cli = Client::new(url, auth);
        cli.ping().await?;
        self.client = Some(cli);
        Ok(())
    }

    pub fn client(&self) -> Result<&Client> {
        self.client
            .as_ref()
            .ok_or_else(|| eyre!("Client not initialized"))
    }

    pub fn current_user(&self) -> Result<&String> {
        self.current_user
            .as_ref()
            .ok_or_else(|| eyre!("Client not initialized"))
    }
}
