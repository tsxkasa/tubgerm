use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use ratatui_notifications::{Notification, Notifications};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    core::event::{AppEvent, Event, NotifLevel, UiCmd},
    ui::{library::LibraryState, login_form::LoginForm, main_view::MainView},
};

#[derive(Default, Debug)]
pub enum UiState {
    #[default]
    Loading,
    Login(LoginForm),
    Main(MainView),
    FatalError(String),
}

#[derive(Debug)]
pub struct Ui {
    state: UiState,
    pub spinner_tick: u8,

    library: LibraryState,
    event_rx: Receiver<AppEvent>,
    command_tx: Sender<UiCmd>,
    notifications: Notifications,
}

impl Ui {
    pub fn new(event_rx: Receiver<AppEvent>, command_tx: Sender<UiCmd>) -> Self {
        Self {
            state: UiState::Loading,
            spinner_tick: 0,
            library: LibraryState::default(),
            event_rx,
            command_tx,
            notifications: Notifications::default(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        match &mut self.state {
            UiState::Loading => {
                let frames = ["|", "/", "-", "\\"];
                let text = format!(
                    "Loading... {}",
                    frames[self.spinner_tick as usize % frames.len()]
                );
                frame.render_widget(
                    Paragraph::new(text)
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL)),
                    frame.area(),
                );
            }
            UiState::Login(form) => {
                form.render(frame);
            }
            UiState::Main(form) => {
                // TODO: PLACEHOLDER
                form.render(frame, &self.library);
            }
            UiState::FatalError(msg) => {
                frame.render_widget(
                    Paragraph::new(format!("Fatal Error: {}", msg))
                        .style(Style::default().fg(Color::Red))
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL)),
                    frame.area(),
                );
            }
        }

        self.notifications.render(frame, frame.area());
    }

    pub async fn handle_event(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::Tick(tick) => {
                self.spinner_tick = self.spinner_tick.wrapping_add(1);
                self.notifications.tick(tick);
            }
            Event::App(e) => {
                self.handle_app_events(e)?;
            }
            Event::Crossterm(e) => match e {
                crossterm::event::Event::Key(k) => {
                    if !self.handle_key(k).await? {
                        return Ok(false);
                    }
                }
                crossterm::event::Event::Resize(_, _) => {}
                _ => {}
            },
        }
        Ok(true)
    }

    fn handle_app_events(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::NeedsLogin { server, username } => {
                self.state = UiState::Login(LoginForm::with_prefill(&server, &username))
            }
            AppEvent::LoginError(e) => {
                if let UiState::Login(form) = &mut self.state {
                    form.error = Some(e);
                } else {
                    self.state = UiState::Login(LoginForm {
                        error: Some(e),
                        ..Default::default()
                    });
                }
            }
            AppEvent::Ready => self.state = UiState::Main(MainView::default()),
            AppEvent::Notify(m, k) => {
                let notif = Notification::new(m);

                let notif = match k {
                    NotifLevel::Info => notif.level(ratatui_notifications::Level::Info),
                    NotifLevel::Warning => notif.level(ratatui_notifications::Level::Warn),
                    NotifLevel::Error => notif.level(ratatui_notifications::Level::Error),
                    NotifLevel::Debug => notif.level(ratatui_notifications::Level::Debug),
                    NotifLevel::Trace => notif.level(ratatui_notifications::Level::Trace),
                }
                .fade(true)
                .slide_direction(ratatui_notifications::SlideDirection::FromTopLeft)
                .timing(
                    ratatui_notifications::Timing::Auto,
                    ratatui_notifications::Timing::Fixed(Duration::from_secs(5)),
                    ratatui_notifications::Timing::Auto,
                )
                .animation(ratatui_notifications::Animation::Slide)
                .margin(1)
                .build()?;

                self.notifications.add(notif)?;
            }
            AppEvent::Error(e) => self.state = UiState::FatalError(e),
            _ => {}
        }
        Ok(())
    }

    async fn handle_key(&mut self, event: KeyEvent) -> Result<bool> {
        if event.modifiers.contains(KeyModifiers::CONTROL)
            && (event.code == KeyCode::Char('c') || event.code == KeyCode::Char('q'))
        {
            self.command_tx.send(UiCmd::Exit).await?;
            // TODO: add interception for <C-c> or <C-q>
            return Ok(false);
        }

        match &mut self.state {
            UiState::Loading => {}
            UiState::Login(form) => {
                if let Some(cmd) = form.handle_key(event) {
                    self.command_tx.send(cmd).await?;
                    self.state = UiState::Loading;
                }
            }
            UiState::Main(form) => {
                if let Some(cmd) = form.handle_key(event, &self.library) {
                    self.command_tx.send(cmd).await?;
                    self.state = UiState::Loading;
                }
            }
            UiState::FatalError(_) => {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn event_rx(&mut self) -> &mut Receiver<AppEvent> {
        &mut self.event_rx
    }

    pub fn command_tx(&mut self) -> &mut Sender<UiCmd> {
        &mut self.command_tx
    }
}
