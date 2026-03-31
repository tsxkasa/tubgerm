use std::sync::Arc;

use color_eyre::eyre::Result;
use submarine::api::get_album_list::Order;
use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

use crate::{
    core::event::{AppEvent, NotifLevel, SongTime, UiCmd},
    services::{
        audio::PlaybackService, client::ClientService, config::config::ConfigService,
        keyring::KeyringService,
    },
};

#[derive(Default, Debug)]
enum AppState {
    #[default]
    Uninitialized,
    NeedsLogin,
    LoggedIn,
    Exited,
}

pub struct App {
    state: AppState,
    event_tx: Sender<AppEvent>,
    command_rx: Receiver<UiCmd>,

    config: Option<ConfigService>,
    client: Option<ClientService>,
    keyring: Option<KeyringService>,
    playback: Option<Arc<Mutex<PlaybackService>>>,
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
            playback: None,
        };

        app.init().await?;
        app.event_loop().await?;
        Ok(())
    }

    #[allow(unused)]
    async fn warn(&self, msg: impl Into<String>) -> Result<()> {
        let _ = self
            .event_tx
            .send(AppEvent::Notify(msg.into(), NotifLevel::Warning))
            .await;
        Ok(())
    }

    #[allow(unused)]
    async fn error(&self, msg: impl Into<String>) -> Result<()> {
        let _ = self
            .event_tx
            .send(AppEvent::Notify(msg.into(), NotifLevel::Error))
            .await;
        Ok(())
    }

    #[allow(unused)]
    async fn info(&self, msg: impl Into<String>) -> Result<()> {
        let _ = self
            .event_tx
            .send(AppEvent::Notify(msg.into(), NotifLevel::Info))
            .await;
        Ok(())
    }

    #[allow(unused)]
    async fn debug(&self, msg: impl Into<String>) -> Result<()> {
        let _ = self
            .event_tx
            .send(AppEvent::Notify(msg.into(), NotifLevel::Debug))
            .await;
        Ok(())
    }

    async fn init(&mut self) -> Result<()> {
        self.client = Some(ClientService::default());
        self.playback = Some(Arc::new(Mutex::new(PlaybackService::new()?)));

        // loop send ProgressTick
        let playback = self.playback.as_ref().unwrap().clone();
        let tx = self.event_tx.clone();
        tokio::spawn(async move {
            loop {
                // inner scope to drop lock before waiting 250ms so UI is more responsive
                {
                    let p = playback.lock().await;
                    if p.is_playing()
                        && let Some(pos) = p.position()
                    {
                        let _ = tx
                            .send(AppEvent::ProgressNow(SongTime {
                                current: pos,
                                end: p.get_end(),
                            }))
                            .await;
                    }
                } // lock drops

                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
        });

        let config = ConfigService::load()?;
        if config.credentials.username.is_empty() || config.credentials.server.is_empty() {
            let _ = self
                .event_tx
                .send(AppEvent::NeedsLogin {
                    server: String::new(),
                    username: String::new(),
                })
                .await;
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
                    let _ = self
                        .event_tx
                        .send(AppEvent::NeedsLogin {
                            server: config.credentials.server.clone(),
                            username: config.credentials.username.clone(),
                        })
                        .await;
                    self.state = AppState::NeedsLogin;
                }
            }
            Ok(None) => {
                self.warn("No saved password found in keyring").await?;
                let _ = self
                    .event_tx
                    .send(AppEvent::NeedsLogin {
                        server: config.credentials.server.clone(),
                        username: config.credentials.username.clone(),
                    })
                    .await;
                self.state = AppState::NeedsLogin;
            }
            Err(e) => {
                // INFO: maybe have to fix this later, idk if i want app breaking err
                let _ = self.event_tx.send(AppEvent::Error(e.to_string())).await;
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
                        let _ = self
                            .event_tx
                            .send(AppEvent::NeedsLogin {
                                server: url,
                                username: uname,
                            })
                            .await;
                    }
                }
                UiCmd::Logout => {
                    self.client = None;
                    // WARN: deletes user credentials consider removing but.
                    self.keyring.as_mut().unwrap().delete_credential()?;
                    self.state = AppState::NeedsLogin;
                    let _ = self
                        .event_tx
                        .send(AppEvent::NeedsLogin {
                            server: String::new(),
                            username: String::new(),
                        })
                        .await;
                }
                UiCmd::Exit => {
                    self.client = None;
                    self.state = AppState::Exited;
                }
                UiCmd::FetchPlaylists => {
                    if let Some(cli) = &self.client {
                        let playlist = cli
                            .client()?
                            .get_playlists(Some(cli.current_user()?))
                            .await?;
                        let _ = self
                            .event_tx
                            .send(AppEvent::PlaylistsLoaded(playlist))
                            .await;
                    }
                }
                UiCmd::FetchPlaylist(id) => {
                    if let Some(cli) = &self.client {
                        let tracks = cli.client()?.get_playlist(id).await?;
                        let _ = self
                            .event_tx
                            .send(AppEvent::PlaylistTracksLoaded(Box::new(tracks)))
                            .await;
                    }
                }
                UiCmd::FetchAlbums => {
                    if let Some(cli) = &self.client {
                        let albums = cli
                            .client()?
                            .get_album_list2(
                                Order::AlphabeticalByName,
                                Some(i16::MAX as usize),
                                None,
                                None::<String>,
                            )
                            .await?;
                        let _ = self.event_tx.send(AppEvent::AlbumsLoaded(albums)).await;
                    }
                }
                UiCmd::FetchAlbum(id) => {
                    if let Some(cli) = &self.client {
                        let tracks = cli.client()?.get_album(id).await?;
                        let _ = self
                            .event_tx
                            .send(AppEvent::AlbumTracksLoaded(Box::new(tracks)))
                            .await;
                    }
                }
                UiCmd::FetchLikedSongs => {
                    if let Some(cli) = &self.client {
                        let tracks = cli.client()?.get_starred2(None::<String>).await?;
                        let _ = self
                            .event_tx
                            .send(AppEvent::LikedSongsLoaded(tracks.song))
                            .await;
                    }
                }
                UiCmd::PlayTrack(id) => {
                    if let Some(cli) = &self.client {
                        let song = cli
                            .client()?
                            .stream(&id, None, None::<String>, None, None::<String>, None, None)
                            .await?;
                        let _ = self
                            .event_tx
                            .send(AppEvent::NowPlaying(Box::new(
                                cli.client()?.get_song(&id).await?,
                            )))
                            .await;
                        if let Some(playback) = &self.playback {
                            playback.lock().await.play_new(song).await?;
                        }
                        let _ = self.event_tx.send(AppEvent::PlaybackResumed).await;
                    }
                }
                UiCmd::Pause => {
                    if let Some(playback) = &self.playback {
                        let playback = playback.lock().await;
                        playback.pause()?;
                    }
                    let _ = self.event_tx.send(AppEvent::PlaybackStopped).await;
                }
                UiCmd::Resume => {
                    if let Some(playback) = &self.playback {
                        let playback = playback.lock().await;
                        playback.play()?;
                    }
                    let _ = self.event_tx.send(AppEvent::PlaybackResumed).await;
                }
                UiCmd::Prev => {}
                UiCmd::Next => {}
                UiCmd::StopTrack => {
                    if let Some(playback) = &self.playback {
                        let playback = playback.lock().await;
                        let _ = playback.stop();
                    }
                }
                UiCmd::SetVolume(v) => {
                    if let Some(playback) = &self.playback {
                        let playback = playback.lock().await;
                        let _ = playback.set_vol(v);
                    }
                    let _ = self.event_tx.send(AppEvent::VolumeChanged(v)).await;
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
                    // TODO: log instead
                    // self.warn(format!("Keyring save failed: {}", e)).await?;
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
                let _ = self.event_tx.send(AppEvent::Ready).await;
            }
            Err(e) => {
                let _ = self
                    .event_tx
                    .send(AppEvent::LoginError(e.to_string()))
                    .await;
            }
        }
        Ok(())
    }
}
