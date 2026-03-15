use color_eyre::eyre::Error;
use keyring::Entry;
use submarine::Client;
use url::Url;
use zeroize::Zeroize;

use crate::core::{fetch::Fetcher, user_data::UserData};

#[derive(Debug)]
pub struct App {
    user_data: UserData,
    client: Client,
}

impl App {
    pub async fn new() -> Result<Self, Error> {
        use std::io::{self, Write};
        let mut fetcher = Fetcher::default();
        let user_data = UserData::load();

        let mut password: String = {
            use rpassword::read_password;
            print!("Enter password: ");
            io::stdout().flush().unwrap();
            read_password().expect("Failed to read <password>")
        };
        password = password.trim().to_string();
        let client = async {
            fetcher
                .create_client(user_data.server(), user_data.username(), password.as_str())
                .await
        }
        .await?;

        // do this after; more time password in raw mem but less contact with OS
        let server = Url::parse(user_data.server())?;
        let service = format!(
            "org.tubgerm.subsonic.server:{}",
            url_normalizer::normalize(server).unwrap().as_str()
        );
        let entry = Entry::new(service.as_str(), user_data.username())?;
        entry.set_password(password.as_str())?;
        password.zeroize();
        Ok(App { user_data, client })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
