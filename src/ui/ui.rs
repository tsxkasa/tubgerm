use color_eyre::eyre::{Error, Result};
use ratatui::Frame;
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
pub struct Ui {
    state: UiState,
    pub spinner_tick: u8,
    command_tx: Sender<UiCmd>,
    event_rx: Receiver<AppEvent>,
}

impl Ui {
    pub fn new(event_rx: Receiver<AppEvent>, command_tx: Sender<UiCmd>) -> Self {
        Self {
            state: UiState::Loading,
            spinner_tick: 0,
            command_tx,
            event_rx,
        }
    }

    pub fn render(&self, frame: &mut Frame) {}

    pub async fn handle_event(&mut self, event: Event) -> Result<bool> {
        todo!()
    }

    pub fn event_rx(&mut self) -> &mut Receiver<AppEvent> {
        &mut self.event_rx
    }

    pub fn command_tx(&mut self) -> &mut Sender<UiCmd> {
        &mut self.command_tx
    }
}
