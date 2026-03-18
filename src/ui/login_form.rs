use crossterm::event::{KeyCode, KeyEvent};
use enum_iterator::{Sequence, next, next_cycle};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};
use ratatui_textarea::TextArea;

use crate::core::event::UiCmd;

#[derive(Default, Debug, PartialEq, Eq, Sequence)]
pub enum LoginField {
    #[default]
    Url,
    Username,
    Password,
    Submit,
}

#[derive(Debug)]
pub struct LoginForm {
    pub server: Box<TextArea<'static>>,
    pub username: Box<TextArea<'static>>,
    pub password: Box<TextArea<'static>>,
    pub focus: LoginField,
    pub error: Option<String>,
}

impl Default for LoginForm {
    fn default() -> Self {
        let mut server = TextArea::default();
        server.set_placeholder_style(Style::default().fg(Color::Gray));
        server.set_placeholder_text("https://server.example");

        let mut username = TextArea::default();
        username.set_placeholder_style(Style::default().fg(Color::Gray));
        username.set_placeholder_text("username");

        let mut password = TextArea::default();
        password.set_placeholder_style(Style::default().fg(Color::Gray));
        password.set_placeholder_text("password");
        password.set_mask_char('\u{2022}');

        server.set_style(Style::default());
        username.set_style(Style::default());
        password.set_style(Style::default());

        Self {
            server: Box::new(server),
            username: Box::new(username),
            password: Box::new(password),
            focus: LoginField::Url,
            error: None,
        }
    }
}

impl LoginForm {
    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(14),
                Constraint::Min(0),
            ])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(vertical[1]);

        let form_area = horizontal[1];

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Server
                Constraint::Length(3), // Username
                Constraint::Length(3), // Password
                Constraint::Length(2), // Submit
                Constraint::Length(1), // Error
            ])
            .split(form_area);

        frame.render_widget(Clear, form_area);
        frame.render_widget(
            Block::default().title(" Login ").borders(Borders::ALL),
            form_area,
        );

        let focused = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let normal = Style::default().fg(Color::White);

        let server_block = Block::default()
            .borders(Borders::ALL)
            .title(" Server URL ")
            .style(if self.focus == LoginField::Url {
                focused
            } else {
                normal
            });

        self.server.set_block(server_block);
        frame.render_widget(&*self.server, chunks[0]);

        let username_block = Block::default()
            .borders(Borders::ALL)
            .title(" Username ")
            .style(if self.focus == LoginField::Username {
                focused
            } else {
                normal
            });

        self.username.set_block(username_block);
        frame.render_widget(&*self.username, chunks[1]);

        let password_block = Block::default()
            .borders(Borders::ALL)
            .title(" Password ")
            .style(if self.focus == LoginField::Password {
                focused
            } else {
                normal
            });

        self.password.set_block(password_block);
        frame.render_widget(&*self.password, chunks[2]);

        let submit_style = if self.focus == LoginField::Submit {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };

        let submit_block = Block::default().borders(Borders::ALL);
        let submit = Paragraph::new(Line::from(" Submit "))
            .alignment(Alignment::Center)
            .style(submit_style)
            .block(submit_block);

        frame.render_widget(submit, chunks[3]);

        if let Some(err) = &self.error {
            frame.render_widget(
                Paragraph::new(err.clone())
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Red)),
                chunks[4],
            );
        } else {
            frame.render_widget(
                Paragraph::new("Tab: move • Enter: submit • Esc: cancel")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Gray)),
                chunks[4],
            );
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<UiCmd> {
        match key.code {
            KeyCode::Tab => {
                // Switch focus
                self.focus = next_cycle(&self.focus);
            }
            KeyCode::Enter => {
                if self.focus == LoginField::Submit {
                    return Some(UiCmd::SubmitLogin {
                        url: self.server.lines().join(""),
                        uname: self.username.lines().join(""),
                        password: self.password.lines().join(""),
                    });
                } else {
                    self.focus = next(&self.focus).unwrap();
                }
            }
            _ => match self.focus {
                LoginField::Url => {
                    self.server.input(key);
                }
                LoginField::Username => {
                    self.username.input(key);
                }
                LoginField::Password => {
                    self.password.input(key);
                }
                LoginField::Submit => {}
            },
        }
        None
    }

    pub fn with_prefill(server: &str, username: &str) -> Self {
        let mut form = Self::default();
        if !server.is_empty() {
            form.server.insert_str(server);
        }
        if !username.is_empty() {
            form.username.insert_str(username);
        }
        form
    }
}
