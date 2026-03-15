use color_eyre::eyre::Error;
use url::Url;
use zeroize::Zeroize;

use crate::services::{client::ClientService, config::ConfigService, keyring::KeyringService};

#[derive(Debug)]
pub struct App {
    config_manager: ConfigService,
    client_manager: ClientService,
    login_manager: KeyringService,
}

impl App {
    pub async fn new() -> Result<Self, Error> {
        // TODO: finish this
        let url = "placeholder".to_string();
        let uname = "placeholder".to_string();
        let mut password = "placeholder".to_string();

        let conf_mgr = ConfigService::new(url.as_str(), uname.as_str());
        password = password.trim().to_string();

        let mut cli_mgr = ClientService::default();
        async {
            cli_mgr
                .create_client(url.as_str(), uname.as_str(), password.as_str())
                .await
        }
        .await?;

        // do this after; more time password in raw mem but less contact with OS
        let server = Url::parse(url.as_str())?;
        let service = format!(
            "org.tubgerm.subsonic.server:{}",
            url_normalizer::normalize(server).unwrap().as_str()
        );
        let login_mgr = KeyringService::new(service.as_str(), uname.as_str())?;
        password.zeroize();
        Ok(App {
            config_manager: conf_mgr,
            client_manager: cli_mgr,
            login_manager: login_mgr,
        })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
