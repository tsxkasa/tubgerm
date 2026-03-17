use color_eyre::eyre::Result;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    core::event::{AppEvent, UiCmd},
    services::{client::ClientService, config::config::ConfigService, keyring::KeyringService},
};

#[derive(Default, Debug)]
enum AppState {
    #[default]
    Uninitialized,
    NeedsLogin,
    LoggedIn,
}

#[derive(Debug)]
pub struct App {
    state: AppState,
    event_tx: Sender<AppEvent>,
    command_rx: Receiver<UiCmd>,

    config: Option<ConfigService>,
    client: Option<ClientService>,
    keyring: Option<KeyringService>,
}

impl App {
    pub async fn run(event_tx: Sender<AppEvent>, command_rx: Receiver<UiCmd>) -> Result<()> {
        let mut app = Self {
            state: AppState::Uninitialized,
            event_tx,
            command_rx,
            config: None,
            client: None,
            keyring: None,
        };

        app.init().await?;
        app.event_loop().await?;
        Ok(())
    }

    async fn init(&mut self) -> Result<()> {
        // init client first, important!!
        self.client = Some(ClientService::default());

        let config = ConfigService::load()?;
        if config.credentials.username.is_empty() || config.credentials.server.is_empty() {
            self.event_tx.send(AppEvent::NeedsLogin).await?;
            self.state = AppState::NeedsLogin;
            return Ok(());
        }

        let keyring = KeyringService::new(
            config.credentials.server.as_str(),
            config.credentials.username.as_str(),
        )?;

        match keyring.get_password() {
            Ok(Some(pw)) => {
                self.try_login(
                    &config.credentials.server,
                    &config.credentials.username,
                    &pw,
                )
                .await?;
            }
            Ok(None) => {
                // Config exists but no password stored
                self.config = Some(ConfigService {});

                // rather set key empty so user can login fresh incase some errors
                ConfigService::set_server("")?;
                ConfigService::set_username("")?;
                self.keyring = Some(keyring);
                self.event_tx.send(AppEvent::NeedsLogin).await?;
                self.state = AppState::NeedsLogin;
            }
            Err(e) => {
                // INFO: maybe have to fix this later, idk if i want app breaking err
                self.event_tx.send(AppEvent::Error(e.to_string())).await?;
            }
        }
        Ok(())
    }

    async fn event_loop(&mut self) -> Result<()> {
        while let Some(cmd) = self.command_rx.recv().await {
            match cmd {
                UiCmd::SubmitLogin {
                    url,
                    uname,
                    password,
                } => {
                    self.try_login(&url, &uname, &password).await?;
                }
                UiCmd::Logout => {
                    self.client = None;
                    self.state = AppState::NeedsLogin;
                    self.event_tx.send(AppEvent::NeedsLogin).await?;
                }
            }
        }
        Ok(())
    }

    async fn try_login(&mut self, server: &str, username: &str, password: &str) -> Result<()> {
        let mut client_svc = ClientService::default();
        match client_svc.create_client(server, username, password).await {
            Ok(()) => {
                // store password, finalize all services
                if self.keyring.is_none() {
                    self.state = AppState::Uninitialized;
                    self.event_tx
                        .send(AppEvent::Error("Weird keyring state: aborting".to_string()))
                        .await?;
                    return Ok(());
                }
                if let Some(k) = &self.keyring {
                    k.set_password(password)?;
                }
                self.config = Some(ConfigService {});
                ConfigService::set_server(server)?;
                ConfigService::set_username(username)?;
                self.client = Some(client_svc);
                self.state = AppState::LoggedIn;
                self.event_tx.send(AppEvent::Ready).await?;
            }
            Err(e) => {
                self.event_tx
                    .send(AppEvent::LoginError(e.to_string()))
                    .await?;
            }
        }
        Ok(())
    }
}
