use std::fmt::Debug;

use color_eyre::eyre::Error;
use submarine::{Client, auth::AuthBuilder};

#[derive(Default, Debug)]
pub struct ClientService {
    client: Option<Client>,
}

impl ClientService {
    pub async fn create_client(&mut self, url: &str, uname: &str, pw: &str) -> Result<(), Error> {
        let auth = AuthBuilder::new(uname, "1.16.1")
            .client_name("org.tubgerm")
            .hashed(pw);
        let cli = Client::new(url, auth);
        cli.ping().await?;
        self.client = Some(cli);
        Ok(())
    }

    pub fn client(&self) -> &Client {
        self.client
            .as_ref()
            .expect("Unable to get client, client may be uninitialized")
    }
}
