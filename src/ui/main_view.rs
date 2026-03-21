use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::core::event::UiCmd;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Focus {
    #[default]
    Main,
    LeftPanel,
    RightPanel,
    Playbar,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum RightPanelKind {
    #[default]
    Queue,
    Lyrics,
    Related,
    ArtistView,
}

impl RightPanelKind {
    const ALL: &'static [RightPanelKind] = &[
        RightPanelKind::Queue,
        RightPanelKind::Lyrics,
        RightPanelKind::Related,
        RightPanelKind::ArtistView,
    ];

    fn label(&self) -> &'static str {
        match self {
            RightPanelKind::Queue => "Queue",
            RightPanelKind::Lyrics => "Lyrics",
            RightPanelKind::Related => "Related",
            RightPanelKind::ArtistView => "Artist view",
        }
    }

    fn next(&self) -> Self {
        let idx = Self::ALL.iter().position(|k| k == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()].clone()
    }

    fn prev(&self) -> Self {
        let idx = Self::ALL.iter().position(|k| k == self).unwrap_or(0);
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()].clone()
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LeftPanelKind {
    #[default]
    Library,
    Playlists,
}

#[derive(Default, Debug)]
pub struct LeftSideState {
    pub kind: LeftPanelKind,
    pub selected: usize,
}

#[derive(Default, Debug)]
pub struct MainPanelState {
    pub selected: usize,
}

#[derive(Default, Debug)]
pub struct RightPanelState {
    pub kind: RightPanelKind,
    pub selected: usize,
}

#[derive(Default, Debug)]
pub struct PlaybarState {
    pub playing: bool,
    pub progress: f64, // 0.0 – 1.0
    pub volume: u8,    // 0 – 100
}

#[derive(Default, Debug)]
pub struct MainView {
    pub focus: Focus,
    pub left: LeftSideState,
    pub main: MainPanelState,
    pub right: RightPanelState,
    pub playbar: PlaybarState,
}

impl MainView {
    fn handle_left(&mut self, key: KeyEvent) -> Option<UiCmd> {
        match key.code {
            KeyCode::Up => {
                self.left.selected = self.left.selected.saturating_sub(1);
            }
            KeyCode::Down => {
                self.left.selected += 1;
            } // TODO: clamp when real data
            KeyCode::Enter => {
                // TODO: navigate into whatever is selected
            }
            _ => {}
        }
        None
    }

    fn handle_main(&mut self, key: KeyEvent) -> Option<UiCmd> {
        match key.code {
            KeyCode::Up => {
                self.main.selected = self.main.selected.saturating_sub(1);
            }
            KeyCode::Down => {
                self.main.selected += 1;
            }
            KeyCode::Enter => {
                // TODO: send UiCmd::Play { .. }
            }
            _ => {}
        }
        None
    }

    fn handle_right(&mut self, key: KeyEvent) -> Option<UiCmd> {
        match key.code {
            KeyCode::Char('[') => {
                self.right.kind = self.right.kind.prev();
            }
            KeyCode::Char(']') => {
                self.right.kind = self.right.kind.next();
            }
            KeyCode::Up => {
                self.right.selected = self.right.selected.saturating_sub(1);
            }
            KeyCode::Down => {
                self.right.selected += 1;
            }
            _ => {}
        }
        None
    }

    fn handle_playbar(&mut self, key: KeyEvent) -> Option<UiCmd> {
        match key.code {
            KeyCode::Left => {
                // seek backward  TODO: send UiCmd::Seek when hooked up
            }
            KeyCode::Right => {
                // seek forward
            }
            KeyCode::Char('-') => {
                self.playbar.volume = self.playbar.volume.saturating_sub(5);
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.playbar.volume = (self.playbar.volume + 5).min(100);
            }
            KeyCode::Char('l') => return Some(UiCmd::Logout),
            _ => {}
        }
        None
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<UiCmd> {
        match key.code {
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::LeftPanel => Focus::Main,
                    Focus::Main => Focus::RightPanel,
                    Focus::RightPanel => Focus::Playbar,
                    Focus::Playbar => Focus::LeftPanel,
                };
                return None;
            }
            KeyCode::BackTab => {
                self.focus = match self.focus {
                    Focus::LeftPanel => Focus::Playbar,
                    Focus::Main => Focus::LeftPanel,
                    Focus::RightPanel => Focus::Main,
                    Focus::Playbar => Focus::RightPanel,
                };
                return None;
            }
            KeyCode::Char(' ') => {
                self.playbar.playing = !self.playbar.playing;
                return None;
            }
            KeyCode::Char('q') => {
                return Some(UiCmd::Exit);
            }
            _ => {}
        }

        match self.focus {
            Focus::LeftPanel => self.handle_left(key),
            Focus::Main => self.handle_main(key),
            Focus::RightPanel => self.handle_right(key),
            Focus::Playbar => self.handle_playbar(key),
        }
    }
}

impl MainView {
    pub fn render(&mut self, frame: &mut Frame) {
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),   // body
                Constraint::Length(4), // 2 lines content + border top/bottom
            ])
            .split(frame.area());

        let body_area = root[0];
        let playbar_area = root[1];

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // left
                Constraint::Fill(1),        // main
                Constraint::Percentage(30), // right
            ])
            .split(body_area);

        let left_area = body[0];
        let main_area = body[1];
        let right_area = body[2];

        self.render_left(frame, left_area);
        self.render_main(frame, main_area);
        self.render_right(frame, right_area);
        self.render_playbar(frame, playbar_area);
    }

    fn render_left(&self, frame: &mut Frame, area: Rect) {
        let focused = self.focus == Focus::LeftPanel;
        let block = block_for("Library", focused);
        frame.render_widget(block, area);

        let inner = inset(area);
        let items = [
            "Library",
            "Playlists",
            "Artists",
            "Albums",
            "──────",
            "Liked Songs",
        ];
        let lines: Vec<Line> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if item.starts_with('─') {
                    Line::from(Span::styled(*item, Style::default().fg(Color::DarkGray)))
                } else if i == self.left.selected && focused {
                    Line::from(Span::styled(
                        format!(" {}", item),
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Green),
                    ))
                } else {
                    Line::from(Span::raw(format!(" {}", item)))
                }
            })
            .collect();

        frame.render_widget(Paragraph::new(lines), inner);
    }

    fn render_main(&self, frame: &mut Frame, area: Rect) {
        let focused = self.focus == Focus::Main;
        let block = block_for("Browse", focused);
        frame.render_widget(block, area);

        let inner = inset(area);
        frame.render_widget(
            Paragraph::new("Placeholder, replace with real content later")
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
    }

    fn render_right(&self, frame: &mut Frame, area: Rect) {
        let focused = self.focus == Focus::RightPanel;

        let title_spans: Vec<Span> = RightPanelKind::ALL
            .iter()
            .flat_map(|kind| {
                let sep = Span::raw(" │ ");
                let label = if kind == &self.right.kind {
                    Span::styled(
                        kind.label(),
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Green),
                    )
                } else {
                    Span::styled(kind.label(), Style::default().fg(Color::DarkGray))
                };
                [label, sep]
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style(focused))
            .title(Line::from(title_spans))
            .title_bottom(Line::from(Span::styled(
                " [ / ] switch panel ",
                Style::default().fg(Color::DarkGray),
            )));

        frame.render_widget(block, area);

        let inner = inset(area);
        let content = match self.right.kind {
            RightPanelKind::Queue => self.render_queue(inner),
            RightPanelKind::Lyrics => Paragraph::new("Lyrics go here..."),
            RightPanelKind::Related => Paragraph::new("Related tracks go here..."),
            RightPanelKind::ArtistView => Paragraph::new("Artist view go here..."),
        };
        frame.render_widget(content, inner);
    }

    fn render_queue(&'_ self, _area: Rect) -> Paragraph<'_> {
        // TODO: replace with a StatefulWidget List
        Paragraph::new(vec![
            Line::from(Span::styled(
                "Now playing",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "Track Name",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled("Artist", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled(
                "Up next",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from("  Track 2"),
            Line::from("  Track 3"),
            Line::from("  Track 4"),
        ])
    }

    fn render_playbar(&self, frame: &mut Frame, area: Rect) {
        let focused = self.focus == Focus::Playbar;
        let block = block_for("", focused);
        frame.render_widget(block, area);

        let inner = inset(area);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // track info
                Constraint::Fill(1),        // controls + progress
                Constraint::Percentage(25), // volume + misc
            ])
            .split(inner);

        // Track info
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    "Track Title",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "Artist Name",
                    Style::default().fg(Color::DarkGray),
                )),
            ]),
            cols[0],
        );

        let play_icon = if self.playbar.playing { "⏸" } else { "▶" };
        let controls = format!("  ⏮  {}  ⏭  ", play_icon);
        // just enough saturating sub to make it look natural
        let bar_width = cols[1].width.saturating_sub(17) as usize;
        let filled = (bar_width as f64 * self.playbar.progress) as usize;
        let progress_bar = format!(
            "0:00 {}{} 0:00",
            "█".repeat(filled),
            "─".repeat(bar_width.saturating_sub(filled)),
        );
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(controls, Style::default().fg(Color::White))),
                Line::from(Span::styled(
                    progress_bar,
                    Style::default().fg(Color::Green),
                )),
            ]),
            cols[1],
        );

        let vol_filled = (10.0 * self.playbar.volume as f64 / 100.0) as usize;
        let vol_bar = format!(
            "vol {}{}",
            "█".repeat(vol_filled),
            "─".repeat(10 - vol_filled)
        );
        frame.render_widget(
            Paragraph::new(vec![Line::from(Span::styled(
                vol_bar,
                Style::default().fg(Color::DarkGray),
            ))])
            .alignment(ratatui::layout::Alignment::Right),
            cols[2],
        );
    }
}

/// Returns a styled Block
fn block_for(title: &'_ str, focused: bool) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(focused))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(if focused {
                Color::Green
            } else {
                Color::DarkGray
            }),
        ))
}

fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

/// Shrinks a Rect by 1 on each side to account for a border.
fn inset(area: Rect) -> Rect {
    Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}
