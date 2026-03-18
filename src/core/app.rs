use color_eyre::eyre::Result;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    core::event::{AppEvent, NotifLevel, UiCmd},
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

    #[allow(unused)]
    async fn warn(&self, msg: impl Into<String>) -> Result<()> {
        self.event_tx
            .send(AppEvent::Notify(msg.into(), NotifLevel::Warning))
            .await?;
        Ok(())
    }

    #[allow(unused)]
    async fn error(&self, msg: impl Into<String>) -> Result<()> {
        self.event_tx
            .send(AppEvent::Notify(msg.into(), NotifLevel::Error))
            .await?;
        Ok(())
    }

    #[allow(unused)]
    async fn info(&self, msg: impl Into<String>) -> Result<()> {
        self.event_tx
            .send(AppEvent::Notify(msg.into(), NotifLevel::Info))
            .await?;
        Ok(())
    }

    #[allow(unused)]
    async fn debug(&self, msg: impl Into<String>) -> Result<()> {
        self.event_tx
            .send(AppEvent::Notify(msg.into(), NotifLevel::Debug))
            .await?;
        Ok(())
    }

    async fn init(&mut self) -> Result<()> {
        // init client first, important!!
        self.client = Some(ClientService::default());

        let config = ConfigService::load()?;
        if config.credentials.username.is_empty() || config.credentials.server.is_empty() {
            self.event_tx
                .send(AppEvent::NeedsLogin {
                    server: String::new(),
                    username: String::new(),
                })
                .await?;
            self.state = AppState::NeedsLogin;
            return Ok(());
        }

        let keyring = KeyringService::new(
            config.credentials.server.as_str(),
            config.credentials.username.as_str(),
        )?;
        self.keyring = Some(keyring);

        match self.keyring.as_ref().unwrap().get_password() {
            Ok(Some(pw)) => {
                if self
                    .try_login(
                        &config.credentials.server,
                        &config.credentials.username,
                        &pw,
                    )
                    .await
                    .is_err()
                {
                    self.warn("Auto-login failed: could not reach server")
                        .await?;
                    self.event_tx
                        .send(AppEvent::NeedsLogin {
                            server: config.credentials.server.clone(),
                            username: config.credentials.username.clone(),
                        })
                        .await?;
                    self.state = AppState::NeedsLogin;
                }
            }
            Ok(None) => {
                self.warn("No saved password found in keyring").await?;
                self.event_tx
                    .send(AppEvent::NeedsLogin {
                        server: config.credentials.server.clone(),
                        username: config.credentials.username.clone(),
                    })
                    .await?;
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
                    if let Err(e) = self.try_login(&url, &uname, &password).await {
                        self.error(format!("Could not login: {}", e)).await?;
                        self.event_tx
                            .send(AppEvent::NeedsLogin {
                                server: url,
                                username: uname,
                            })
                            .await?;
                    }
                }
                UiCmd::Logout => {
                    self.client = None;
                    self.state = AppState::NeedsLogin;
                    self.event_tx
                        .send(AppEvent::NeedsLogin {
                            server: String::new(),
                            username: String::new(),
                        })
                        .await?;
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
                    match KeyringService::new(server, username) {
                        Ok(new_keyring) => self.keyring = Some(new_keyring),
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                if let Some(k) = &self.keyring
                    && let Err(e) = k.set_password(password)
                {
                    self.warn(format!("Keyring save failed: {}", e)).await?;
                }

                self.config = Some(ConfigService::new()?);
                self.config
                    .as_mut()
                    .unwrap()
                    .set_credentials(server, username)?;
                if let Err(e) = self.config.as_mut().unwrap().save() {
                    self.warn(format!("Failed to save config: {}", e)).await?;
                }
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
