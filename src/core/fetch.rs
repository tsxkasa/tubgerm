use std::fmt::Debug;

use color_eyre::eyre::Error;
use submarine::{Client, SubsonicError, auth::AuthBuilder, data::Info};

#[derive(Default, Debug)]
pub struct Fetcher {}

impl Fetcher {
    pub async fn create_client(
        &mut self,
        url: &str,
        uname: &str,
        pw: &str,
    ) -> Result<Client, Error> {
        let auth = AuthBuilder::new(uname, "1.16.1")
            .client_name("org.tubgerm")
            .hashed(pw);
        let cli = Client::new(url, auth);
        let result = cli.ping().await;
        match result {
            Ok(_) => {
                print!("GOT REPLY OMG ><");
            }
            Err(e) => {
                print!("Uh OH!!! NO REPLY\n {}", e);
            }
        };
        Ok(cli)
    }
}
