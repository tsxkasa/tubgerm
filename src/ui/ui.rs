use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    core::event::{AppEvent, Event, UiCmd},
    ui::{login_form::LoginForm, main_view::MainView},
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
pub enum NotifKind {
    Info,
    Warning,
    Error,
}

#[derive(Debug)]
pub struct Ui {
    state: UiState,
    pub spinner_tick: u8,
    command_tx: Sender<UiCmd>,
}

impl Ui {
    pub fn new(command_tx: Sender<UiCmd>) -> Self {
        Self {
            state: UiState::Loading,
            spinner_tick: 0,
            command_tx,
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        match &self.state {
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
                frame.render_widget(
                    Paragraph::new("Main App View - Logged In!")
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL)),
                    frame.area(),
                );
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
    }

    pub async fn handle_event(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::Tick => {
                self.spinner_tick = self.spinner_tick.wrapping_add(1);
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
            AppEvent::NeedsLogin => self.state = UiState::Login(LoginForm::default()),
            AppEvent::LoginError(e) => {
                if let UiState::Login(form) = &mut self.state {
                    form.error = Some(e);
                } else {
                    let mut form = LoginForm::default();
                    form.error = Some(e);
                    self.state = UiState::Login(form);
                }
            }
            AppEvent::Ready => self.state = UiState::Main(MainView::default()),
            AppEvent::Notify(m, k) => {}
            AppEvent::Error(e) => self.state = UiState::FatalError(e),
        }
        Ok(())
    }

    async fn handle_key(&mut self, event: KeyEvent) -> Result<bool> {
        if event.modifiers.contains(KeyModifiers::CONTROL)
            && (event.code == KeyCode::Char('c') || event.code == KeyCode::Char('q'))
        {
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
            UiState::Main(form) => {}
            UiState::FatalError(_) => {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn command_tx(&mut self) -> &mut Sender<UiCmd> {
        &mut self.command_tx
    }
}
