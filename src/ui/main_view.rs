use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Alignment,
    widgets::{Block, Borders, Paragraph},
};

use crate::core::event::UiCmd;

#[derive(Default, Debug)]
enum Panel {
    #[default]
    PLACEHOLDER,
}

#[derive(Default, Debug)]
pub struct MainView {
    focus: Panel,
}

impl MainView {
    pub fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(
            Paragraph::new("Main App View - Logged In!")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL)),
            frame.area(),
        );
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<UiCmd> {
        match key.code {
            KeyCode::Char('l') => Some(UiCmd::Logout),
            _ => None,
        }
    }
}
