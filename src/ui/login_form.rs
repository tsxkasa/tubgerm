use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph},
};
use ratatui_textarea::TextArea;

use crate::core::event::UiCmd;

#[derive(Default, Debug, PartialEq, Eq)]
pub enum LoginField {
    #[default]
    Url,
    Username,
    Password,
    Submit,
}

#[derive(Default, Debug)]
pub struct LoginForm {
    pub server: String,
    pub username: String,
    pub password: String,
    pub focus: LoginField,
    pub error: Option<String>,
}

impl LoginForm {
    pub fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // URL
                Constraint::Length(3), // Username
                Constraint::Length(3), // Password
                Constraint::Length(1), // Error line
            ])
            .split(frame.area());

        let url_style = if self.focus == LoginField::Url {
            ratatui::style::Style::default().fg(ratatui::style::Color::LightMagenta)
        } else {
            ratatui::style::Style::default()
        };

        frame.render_widget(
            Paragraph::new(self.server.as_str()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Server URL")
                    .style(url_style),
            ),
            chunks[0],
        );

        let username_style = if self.focus == LoginField::Username {
            ratatui::style::Style::default().fg(ratatui::style::Color::Cyan)
        } else {
            ratatui::style::Style::default()
        };
        frame.render_widget(
            Paragraph::new(self.username.as_str()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Username")
                    .style(username_style),
            ),
            chunks[1],
        );

        let password_style = if self.focus == LoginField::Password {
            ratatui::style::Style::default().fg(ratatui::style::Color::LightBlue)
        } else {
            ratatui::style::Style::default()
        };
        frame.render_widget(
            Paragraph::new(self.password.as_str()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Password")
                    .style(password_style),
            ),
            chunks[2],
        );
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<UiCmd> {
        match key.code {
            KeyCode::Tab => {
                // Switch focus
                self.focus = match self.focus {
                    LoginField::Url => LoginField::Username,
                    LoginField::Username => LoginField::Password,
                    LoginField::Password => LoginField::Submit,
                    LoginField::Submit => LoginField::Url,
                };
            }
            KeyCode::Enter => {
                if matches!(self.focus, LoginField::Submit)
                    || (!self.server.is_empty()
                        && !self.username.is_empty()
                        && !self.password.is_empty())
                {
                    return Some(UiCmd::SubmitLogin {
                        url: self.server.clone(),
                        uname: self.username.clone(),
                        password: self.password.clone(),
                    });
                }
            }
            KeyCode::Char(c) => match self.focus {
                LoginField::Url => self.server.push(c),
                LoginField::Username => self.username.push(c),
                LoginField::Password => self.password.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.focus {
                LoginField::Url => {
                    self.server.pop();
                }
                LoginField::Username => {
                    self.username.pop();
                }
                LoginField::Password => {
                    self.password.pop();
                }
                _ => {}
            },
            _ => {}
        }
        None
    }
}
