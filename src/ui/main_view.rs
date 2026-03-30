use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState, Wrap,
    },
};
use submarine::data::Child;

use crate::{core::event::UiCmd, ui::library::LibraryState};

const PLACEHOLDER_LYRICS: &str = "\
    position
    position
    position
    position
    position
    position
    position
    position
    position
    position
    position
    position
    position
    position
    position
    position
    position
    position
    position
";

const LYRICS_LINE_COUNT: u16 = 19;

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
}

impl RightPanelKind {
    const ALL: &'static [RightPanelKind] = &[
        RightPanelKind::Queue,
        RightPanelKind::Lyrics,
        RightPanelKind::Related,
    ];

    fn label(&self) -> &'static str {
        match self {
            RightPanelKind::Queue => "Queue",
            RightPanelKind::Lyrics => "Lyrics",
            RightPanelKind::Related => "Related",
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

#[derive(Clone, Debug, PartialEq)]
pub enum SidebarTarget {
    LikedSongs,
    Albums,
    Playlists,
    Playlist(usize),
}

#[derive(Debug)]
enum SidebarRow {
    Header(&'static str),
    Nav {
        label: &'static str,
        target: SidebarTarget,
    },
}

#[derive(Debug)]
pub struct LeftSideState {
    static_rows: Vec<SidebarRow>,
    list_state: ListState,
}

impl Default for LeftSideState {
    fn default() -> Self {
        let static_rows = vec![
            SidebarRow::Header("Browse"),
            SidebarRow::Nav {
                label: "Liked Songs",
                target: SidebarTarget::LikedSongs,
            },
            SidebarRow::Nav {
                label: "Albums",
                target: SidebarTarget::Albums,
            },
            SidebarRow::Nav {
                label: "Playlists",
                target: SidebarTarget::Playlists,
            },
            SidebarRow::Header("Your Playlists"),
        ];
        let mut list_state = ListState::default();
        list_state.select(Some(1)); // first Nav row
        Self {
            static_rows,
            list_state,
        }
    }
}

impl LeftSideState {
    fn selected_idx(&self) -> usize {
        self.list_state.selected().unwrap_or(1)
    }

    fn row_is_selectable(&self, idx: usize, lib: &LibraryState) -> bool {
        if idx < self.static_rows.len() {
            matches!(self.static_rows[idx], SidebarRow::Nav { .. })
        } else {
            idx - self.static_rows.len() < lib.playlists.as_ref().map(|v| v.len()).unwrap_or(0)
        }
    }

    fn select_next(&mut self, lib: &LibraryState) {
        let cur = self.selected_idx();
        let total = self.static_rows.len() + lib.playlists.as_ref().map(|v| v.len()).unwrap_or(0);
        if let Some(i) = ((cur + 1)..total).find(|&i| self.row_is_selectable(i, lib)) {
            self.list_state.select(Some(i));
        }
    }

    fn select_prev(&mut self, lib: &LibraryState) {
        let cur = self.selected_idx();
        if let Some(i) = (0..cur).rev().find(|&i| self.row_is_selectable(i, lib)) {
            self.list_state.select(Some(i));
        }
    }

    fn selected_target(&self, lib: &LibraryState) -> Option<SidebarTarget> {
        let idx = self.selected_idx();
        if idx < self.static_rows.len() {
            match &self.static_rows[idx] {
                SidebarRow::Nav { target, .. } => Some(target.clone()),
                SidebarRow::Header(_) => None,
            }
        } else {
            let playlist_idx = idx - self.static_rows.len();
            (playlist_idx < lib.playlists.as_ref().map(|v| v.len()).unwrap_or(0))
                .then_some(SidebarTarget::Playlist(playlist_idx))
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum MainContent {
    #[default]
    Albums,
    Playlists,
    LikedSongs,
    Playlist(String),
    Album(String),
}

impl MainContent {
    pub fn panel_title<'a>(&self, lib: &'a LibraryState) -> &'a str {
        match self {
            MainContent::Albums => "Albums",
            MainContent::Playlists => "Playlists",
            MainContent::LikedSongs => "Liked Songs",
            MainContent::Playlist(id) => lib
                .playlist_cache
                .get(id)
                .map(|p| p.base.name.as_str())
                .unwrap_or("Playlist"),
            MainContent::Album(id) => lib
                .album_cache
                .get(id)
                .map(|a| a.base.name.as_str())
                .unwrap_or("Album"),
        }
    }

    pub fn current_tracks<'a>(&self, lib: &'a LibraryState) -> &'a [Child] {
        match self {
            MainContent::LikedSongs => lib.liked_songs.as_deref().unwrap_or(&[]),
            MainContent::Playlist(id) => lib
                .playlist_cache
                .get(id)
                .map(|p| p.entry.as_slice())
                .unwrap_or(&[]),
            MainContent::Album(id) => lib
                .album_cache
                .get(id)
                .map(|a| a.song.as_slice())
                .unwrap_or(&[]),
            _ => &[],
        }
    }

    pub fn len(&self, lib: &LibraryState) -> usize {
        match self {
            MainContent::Albums => lib.albums.as_ref().map(|v| v.len()).unwrap_or(0),
            MainContent::Playlists => lib.playlists.as_ref().map(|v| v.len()).unwrap_or(0),
            MainContent::LikedSongs | MainContent::Playlist(_) | MainContent::Album(_) => {
                self.current_tracks(lib).len()
            }
        }
    }
}

#[derive(Debug)]
pub struct MainPanelState {
    pub content: MainContent,
    pub table_state: TableState,
}

impl Default for MainPanelState {
    fn default() -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Self {
            content: MainContent::default(),
            table_state,
        }
    }
}

impl MainPanelState {
    fn select_next(&mut self, lib: &LibraryState) {
        let max = self.content.len(lib).saturating_sub(1);
        let cur = self.table_state.selected().unwrap_or(0);
        self.table_state.select(Some((cur + 1).min(max)));
    }

    fn select_prev(&mut self) {
        let cur = self.table_state.selected().unwrap_or(0);
        self.table_state.select(Some(cur.saturating_sub(1)));
    }
}

#[derive(Debug, Default)]
pub struct RightPanelState {
    pub kind: RightPanelKind,
    pub tab_offset: usize,
    pub queue_list_state: ListState,
    pub lyrics_scroll: u16,
    pub lyrics_visible_height: u16,
    pub lyrics_timed: bool,
    pub related_state: ListState,
}

#[derive(Default, Debug)]
pub struct MainView {
    pub focus: Focus,
    pub left: LeftSideState,
    pub main: MainPanelState,
    pub right: RightPanelState,
}

impl MainView {
    pub fn handle_key(&mut self, key: KeyEvent, lib: &LibraryState) -> Option<UiCmd> {
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
                return Some(if lib.playing {
                    UiCmd::Pause
                } else {
                    UiCmd::Resume
                });
            }
            _ => {}
        }

        match self.focus {
            Focus::LeftPanel => self.handle_left(key, lib),
            Focus::Main => self.handle_main(key, lib),
            Focus::RightPanel => self.handle_right(key, lib),
            Focus::Playbar => self.handle_playbar(key, lib),
        }
    }

    fn handle_left(&mut self, key: KeyEvent, lib: &LibraryState) -> Option<UiCmd> {
        match key.code {
            KeyCode::Char('j') => self.left.select_next(lib),
            KeyCode::Char('k') => self.left.select_prev(lib),
            KeyCode::Char('l') | KeyCode::Enter => {
                if let Some(target) = self.left.selected_target(lib) {
                    self.focus = Focus::Main;
                    return self.navigate_main(target, lib);
                }
            }
            _ => {}
        }
        None
    }

    fn navigate_main(&mut self, target: SidebarTarget, lib: &LibraryState) -> Option<UiCmd> {
        let mut ts = TableState::default();
        ts.select(Some(0));
        self.main.table_state = ts;

        match target {
            SidebarTarget::LikedSongs => {
                self.main.content = MainContent::LikedSongs;
                match (&lib.liked_songs, &lib.liked_cache) {
                    (Some(songs), cache) if !songs.is_empty() || !cache.is_empty() => None,
                    _ => Some(UiCmd::FetchLikedSongs),
                }
            }
            SidebarTarget::Albums => {
                self.main.content = MainContent::Albums;
                match (&lib.albums, &lib.album_cache) {
                    (Some(albums), cache) if !albums.is_empty() || !cache.is_empty() => None,
                    _ => Some(UiCmd::FetchAlbums),
                }
            }
            SidebarTarget::Playlists => {
                self.main.content = MainContent::Playlists;
                None
            }
            SidebarTarget::Playlist(idx) => {
                if let Some(playlists) = &lib.playlists {
                    if let Some(p) = playlists.get(idx) {
                        let id = p.id.clone();
                        self.main.content = MainContent::Playlist(id.clone());
                        (!lib.playlist_cache.contains_key(&id)).then_some(UiCmd::FetchPlaylist(id))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    fn handle_main(&mut self, key: KeyEvent, lib: &LibraryState) -> Option<UiCmd> {
        match key.code {
            KeyCode::Char('j') => self.main.select_next(lib),
            KeyCode::Char('k') => self.main.select_prev(),
            KeyCode::Char('h') => {
                return match &self.main.content {
                    MainContent::Album(_) => {
                        self.main.content = MainContent::Albums;
                        None
                    }
                    MainContent::Playlist(_) => {
                        self.main.content = MainContent::Playlists;
                        None
                    }
                    _ => None,
                };
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                let idx = self.main.table_state.selected().unwrap_or(0);
                return match &self.main.content {
                    MainContent::Albums => {
                        if let Some(a) = &lib.albums {
                            let a = a.get(idx)?;
                            let id = a.id.clone();
                            self.main.content = MainContent::Album(id.clone());
                            let mut ts = TableState::default();
                            ts.select(Some(0));
                            self.main.table_state = ts;
                            (!lib.album_cache.contains_key(&id)).then_some(UiCmd::FetchAlbum(id))
                        } else {
                            None
                        }
                    }
                    MainContent::Playlists => {
                        if let Some(p) = &lib.playlists {
                            let p = p.get(idx)?;
                            let id = p.id.clone();
                            self.main.content = MainContent::Playlist(id.clone());
                            let mut ts = TableState::default();
                            ts.select(Some(0));
                            self.main.table_state = ts;
                            (!lib.playlist_cache.contains_key(&id))
                                .then_some(UiCmd::FetchPlaylist(id))
                        } else {
                            None
                        }
                    }
                    MainContent::LikedSongs | MainContent::Playlist(_) | MainContent::Album(_) => {
                        self.main
                            .content
                            .current_tracks(lib)
                            .get(idx)
                            .map(|t| UiCmd::PlayTrack(t.id.clone()))
                    }
                };
            }
            _ => {}
        }
        None
    }

    fn handle_right(&mut self, key: KeyEvent, lib: &LibraryState) -> Option<UiCmd> {
        match key.code {
            KeyCode::Char('[') => {
                self.right.kind = self.right.kind.prev();
                if self.right.kind == RightPanelKind::Related {
                    self.right.related_state.select(Some(0));
                }
            }
            KeyCode::Char(']') => {
                self.right.kind = self.right.kind.next();
                if self.right.kind == RightPanelKind::Related {
                    self.right.related_state.select(Some(0));
                }
            }
            KeyCode::Char('j') => match self.right.kind {
                RightPanelKind::Queue => {
                    let max = lib.queue.len().saturating_sub(1);
                    let cur = self.right.queue_list_state.selected().unwrap_or(0);
                    self.right.queue_list_state.select(Some((cur + 1).min(max)));
                }
                RightPanelKind::Lyrics => {
                    if self.right.lyrics_timed {
                        let max =
                            LYRICS_LINE_COUNT.saturating_sub(self.right.lyrics_visible_height);
                        self.right.lyrics_scroll = (self.right.lyrics_scroll + 1).min(max);
                    }
                }
                RightPanelKind::Related => {
                    let max = lib.related_tracks.len().saturating_sub(1);
                    let cur = self.right.related_state.selected().unwrap_or(0);
                    self.right.related_state.select(Some((cur + 1).min(max)));
                }
            },
            KeyCode::Char('k') => match self.right.kind {
                RightPanelKind::Queue => {
                    let cur = self.right.queue_list_state.selected().unwrap_or(0);
                    self.right
                        .queue_list_state
                        .select(Some(cur.saturating_sub(1)));
                }
                RightPanelKind::Lyrics => {
                    if self.right.lyrics_timed {
                        self.right.lyrics_scroll = self.right.lyrics_scroll.saturating_sub(1);
                    }
                }
                RightPanelKind::Related => {
                    let cur = self.right.related_state.selected().unwrap_or(0);
                    self.right.related_state.select(Some(cur.saturating_sub(1)));
                }
            },
            _ => {}
        }
        None
    }

    fn handle_playbar(&self, key: KeyEvent, lib: &LibraryState) -> Option<UiCmd> {
        match key.code {
            KeyCode::Char('-') => Some(UiCmd::SetVolume((lib.volume - 0.05).clamp(0.0, 100.0))),
            KeyCode::Char('+') | KeyCode::Char('=') => {
                Some(UiCmd::SetVolume((lib.volume + 0.05).clamp(0.0, 100.0)))
            }
            KeyCode::Char('n') => Some(UiCmd::Next),
            KeyCode::Char('p') => Some(UiCmd::Prev),
            KeyCode::Char('l') => Some(UiCmd::Logout),
            _ => None,
        }
    }
}

impl MainView {
    pub fn render(&mut self, frame: &mut Frame, lib: &LibraryState) {
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(4)])
            .split(frame.area());

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(15),
                Constraint::Fill(1),
                Constraint::Percentage(20),
            ])
            .split(root[0]);

        self.render_left(frame, body[0], lib);
        self.render_main(frame, body[1], lib);
        self.render_right(frame, body[2], lib);
        self.render_playbar(frame, root[1], lib);
    }

    fn render_left(&mut self, frame: &mut Frame, area: Rect, lib: &LibraryState) {
        let focused = self.focus == Focus::LeftPanel;

        let mut items: Vec<ListItem> = self
            .left
            .static_rows
            .iter()
            .map(|row| match row {
                SidebarRow::Header(label) => ListItem::new(Line::from(Span::styled(
                    format!(" {} ", label),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ))),
                SidebarRow::Nav { label, .. } => {
                    ListItem::new(Line::from(Span::raw(format!("  {}", label))))
                }
            })
            .collect();

        if let Some(playlists) = &lib.playlists {
            for p in playlists.iter() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {}", p.name),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("  {}", p.song_count),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::DIM),
                    ),
                ])));
            }
        }

        let list = List::new(items)
            .block(block_for("Library", focused))
            .highlight_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("►");

        frame.render_stateful_widget(list, area, &mut self.left.list_state);
    }

    fn render_main(&mut self, frame: &mut Frame, area: Rect, lib: &LibraryState) {
        let focused = self.focus == Focus::Main;
        let title = self.main.content.panel_title(lib).to_string();
        let inner = inset(area);
        let sel = self.main.table_state.selected().unwrap_or(0);
        let total = self.main.content.len(lib);
        let visible_rows = inner.height.saturating_sub(2) as usize;

        let hl = Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD);
        let hdr = Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD);

        match &self.main.content {
            MainContent::Albums => {
                let mut rows = Vec::new();
                if let Some(albums) = &lib.albums {
                    rows = albums
                        .iter()
                        .map(|a| {
                            Row::new(vec![
                                Cell::from(a.name.as_str()),
                                Cell::from(a.artist.as_deref().unwrap_or("—")),
                                Cell::from(a.year.map(|y| y.to_string()).unwrap_or_default()),
                            ])
                        })
                        .collect();
                }
                rows.push(Row::new([""; 4]));
                let table = Table::new(
                    rows,
                    [
                        Constraint::Fill(2),
                        Constraint::Fill(2),
                        Constraint::Length(6),
                    ],
                )
                .header(
                    Row::new(["Title", "Artist", "Year"])
                        .style(hdr)
                        .bottom_margin(1),
                )
                .block(block_for(&title, focused))
                .row_highlight_style(hl)
                .highlight_symbol("► ");
                frame.render_stateful_widget(table, area, &mut self.main.table_state);
            }

            MainContent::Playlists => {
                let mut rows = Vec::new();
                if let Some(playlists) = &lib.playlists {
                    rows = playlists
                        .iter()
                        .map(|p| {
                            Row::new(vec![
                                Cell::from(p.name.as_str()),
                                Cell::from(p.song_count.to_string()),
                                Cell::from(fmt_duration_i32(p.duration)),
                            ])
                        })
                        .collect();
                }
                rows.push(Row::new([""; 3]));
                let table = Table::new(
                    rows,
                    [
                        Constraint::Fill(1),
                        Constraint::Length(8),
                        Constraint::Length(7),
                    ],
                )
                .header(
                    Row::new(["Name", "Tracks", "Length"])
                        .style(hdr)
                        .bottom_margin(1),
                )
                .block(block_for(&title, focused))
                .row_highlight_style(hl)
                .highlight_symbol("► ");
                frame.render_stateful_widget(table, area, &mut self.main.table_state);
            }

            MainContent::LikedSongs | MainContent::Playlist(_) | MainContent::Album(_) => {
                let mut rows: Vec<Row> = self
                    .main
                    .content
                    .current_tracks(lib)
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        Row::new(vec![
                            Cell::from(format!("{:>2}", i + 1)),
                            Cell::from(t.title.as_str()),
                            Cell::from(t.artist.as_deref().unwrap_or("—")),
                            Cell::from(t.album.as_deref().unwrap_or("—")),
                            Cell::from(fmt_duration(t.duration)),
                        ])
                    })
                    .collect();
                rows.push(Row::new([""; 5]));
                let table = Table::new(
                    rows,
                    [
                        Constraint::Length(3),
                        Constraint::Fill(2),
                        Constraint::Fill(2),
                        Constraint::Fill(2),
                        Constraint::Length(5),
                    ],
                )
                .header(
                    Row::new(["#", "Title", "Artist", "Album", ""])
                        .style(hdr)
                        .bottom_margin(1),
                )
                .block(block_for(&title, focused))
                .row_highlight_style(hl)
                .highlight_symbol("► ");
                frame.render_stateful_widget(table, area, &mut self.main.table_state);
            }
        }

        if total > visible_rows {
            let mut sb = ScrollbarState::new(total).position(sel);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .style(Style::default().fg(Color::DarkGray)),
                inner,
                &mut sb,
            );
        }
    }

    fn render_right(&mut self, frame: &mut Frame, area: Rect, lib: &LibraryState) {
        let focused = self.focus == Focus::RightPanel;
        let all = RightPanelKind::ALL;
        let active_idx = all.iter().position(|k| k == &self.right.kind).unwrap_or(0);

        let available = area.width.saturating_sub(4) as usize;
        let left_ellipsis = if self.right.tab_offset > 0 { 4 } else { 0 };
        let mut used = left_ellipsis;
        let mut vis = 0usize;

        for kind in &all[self.right.tab_offset..] {
            let cost = kind.label().len() + 3;
            let right = if self.right.tab_offset + vis + 1 < all.len() {
                4
            } else {
                0
            };
            if used + cost + right > available {
                break;
            }
            used += cost;
            vis += 1;
        }
        let vis = vis.max(1);

        if active_idx < self.right.tab_offset {
            self.right.tab_offset = active_idx;
        } else if active_idx >= self.right.tab_offset + vis {
            self.right.tab_offset = active_idx.saturating_sub(vis - 1);
        }

        let vis_end = (self.right.tab_offset + vis).min(all.len());
        let mut title_spans: Vec<Span> = vec![Span::raw(" ")];

        if self.right.tab_offset > 0 {
            title_spans.push(Span::styled("... │ ", Style::default().fg(Color::DarkGray)));
        }

        let visible_slice = &all[self.right.tab_offset..vis_end];
        for (i, kind) in visible_slice.iter().enumerate() {
            let is_active = kind == &self.right.kind;
            title_spans.push(if is_active {
                Span::styled(
                    kind.label(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(kind.label(), Style::default().fg(Color::DarkGray))
            });
            if i < visible_slice.len() - 1 {
                title_spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
            } else {
                title_spans.push(Span::raw(" "));
            }
        }

        if vis_end < all.len() {
            title_spans.push(Span::styled("...", Style::default().fg(Color::DarkGray)));
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style(focused))
            .title(Line::from(title_spans))
            .title_bottom(Line::from(Span::styled(
                " [/] tab ",
                Style::default().fg(Color::DarkGray),
            )));

        frame.render_widget(block, area);
        let inner = inset(area);

        let kind = self.right.kind.clone();
        match kind {
            RightPanelKind::Queue => self.render_queue(frame, inner, lib),
            RightPanelKind::Lyrics => self.render_lyrics(frame, inner),
            RightPanelKind::Related => self.render_related(frame, inner, lib),
        }
    }

    fn render_queue(&mut self, frame: &mut Frame, area: Rect, lib: &LibraryState) {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(area);

        {
            let content = if let Some(np) = lib.now_playing.as_ref() {
                vec![
                    Line::from(Span::styled(
                        "Now Playing",
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        np.title.as_str(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        format!(
                            "{}  ·  {}",
                            np.artist.as_deref().unwrap_or("—"),
                            np.album.as_deref().unwrap_or("—")
                        ),
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(Span::styled(
                        fmt_duration(np.duration),
                        Style::default().fg(Color::Green),
                    )),
                ]
            } else {
                vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "Nothing playing",
                        Style::default().fg(Color::DarkGray),
                    )),
                ]
            };
            frame.render_widget(Paragraph::new(content), sections[0]);
        }

        frame.render_widget(
            Paragraph::new(Span::styled(
                "─".repeat(area.width.saturating_sub(2) as usize),
                Style::default().fg(Color::DarkGray),
            )),
            sections[1],
        );

        let title_w = sections[2].width.saturating_sub(8) as usize;
        let items: Vec<ListItem> = lib
            .queue
            .iter()
            .map(|t| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{:<width$}", t.title, width = title_w.min(28)),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        fmt_duration(t.duration),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect();

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");
        frame.render_stateful_widget(list, sections[2], &mut self.right.queue_list_state);
    }

    fn render_lyrics(&mut self, frame: &mut Frame, area: Rect) {
        self.right.lyrics_visible_height = area.height;

        if self.right.lyrics_timed {
            let max_scroll = LYRICS_LINE_COUNT.saturating_sub(area.height);
            self.right.lyrics_scroll = self.right.lyrics_scroll.min(max_scroll);

            let content_area = Rect {
                height: area.height.saturating_sub(1),
                ..area
            };
            let indicator_area = Rect {
                y: area.y + area.height.saturating_sub(1),
                height: 1,
                ..area
            };

            frame.render_widget(
                Paragraph::new(PLACEHOLDER_LYRICS)
                    .wrap(Wrap { trim: false })
                    .scroll((self.right.lyrics_scroll, 0))
                    .style(Style::default().fg(Color::DarkGray)),
                content_area,
            );

            let current = (self.right.lyrics_scroll + area.height).min(LYRICS_LINE_COUNT);
            let pct = (current as f32 / LYRICS_LINE_COUNT as f32 * 100.0) as u16;
            frame.render_widget(
                Paragraph::new(Span::styled(
                    format!("line {}/{} ({}%)", current, LYRICS_LINE_COUNT, pct),
                    Style::default().fg(Color::DarkGray),
                ))
                .alignment(Alignment::Right),
                indicator_area,
            );
        } else {
            frame.render_widget(
                Paragraph::new(PLACEHOLDER_LYRICS)
                    .wrap(Wrap { trim: false })
                    .style(Style::default().fg(Color::DarkGray)),
                area,
            );
        }
    }

    fn render_related(&mut self, frame: &mut Frame, area: Rect, lib: &LibraryState) {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Fill(1)])
            .split(area);

        {
            let title = match lib.now_playing.as_ref() {
                Some(t) => t.title.as_str(),
                None => "—",
            };
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled(
                        format!("Similar to: {}", title),
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(Span::styled(
                        "─".repeat(area.width.saturating_sub(2) as usize),
                        Style::default().fg(Color::DarkGray),
                    )),
                ]),
                sections[0],
            );
        }

        let items: Vec<ListItem> = lib
            .related_tracks
            .iter()
            .map(|t| {
                ListItem::new(vec![
                    Line::from(Span::styled(
                        format!("  {}", t.title),
                        Style::default().fg(Color::White),
                    )),
                    Line::from(Span::styled(
                        format!("    {}", t.artist.as_deref().unwrap_or("—")),
                        Style::default().fg(Color::DarkGray),
                    )),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("►");
        frame.render_stateful_widget(list, sections[1], &mut self.right.related_state);
    }

    fn render_playbar(&self, frame: &mut Frame, area: Rect, lib: &LibraryState) {
        let focused = self.focus == Focus::Playbar;
        frame.render_widget(block_for("", focused), area);
        let inner = inset(area);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Fill(1),
                Constraint::Percentage(25),
            ])
            .split(inner);

        if let Some(t) = lib.now_playing.as_ref() {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled(
                        t.title.as_str(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        format!(
                            "{}  ·  {}",
                            t.artist.as_deref().unwrap_or("—"),
                            t.album.as_deref().unwrap_or("—")
                        ),
                        Style::default().fg(Color::DarkGray),
                    )),
                ]),
                cols[0],
            );
        }

        let play = if lib.playing { "⏸" } else { "▶" };
        let bar_w = cols[1].width.saturating_sub(17) as usize;
        let filled = ((bar_w as f64) * lib.progress) as usize;
        let elapsed = match lib.now_playing.as_ref() {
            Some(t) => match t.duration {
                Some(s) => {
                    let e = (s as f64 * lib.progress) as i32;
                    format!("{}:{:02}", e / 60, e % 60)
                }
                None => "--:--".to_string(),
            },
            None => "--:--".to_string(),
        };
        let total_str = match lib.now_playing.as_ref() {
            Some(t) => fmt_duration(t.duration),
            None => "--:--".to_string(),
        };

        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    format!("⏮  {}  ⏭", play),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::styled(
                    format!(
                        "{} {}{} {}",
                        elapsed,
                        "█".repeat(filled),
                        "─".repeat(bar_w.saturating_sub(filled)),
                        total_str
                    ),
                    Style::default().fg(Color::Green),
                )),
            ])
            .alignment(Alignment::Center),
            cols[1],
        );

        let vol_w = 10usize;
        // INFO: volume should be normalized in 0.0 - 1.0
        let vol_f = ((vol_w as f64) * lib.volume) as usize;
        frame.render_widget(
            Paragraph::new(vec![Line::from(vec![
                Span::styled("vol  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(
                        "{}{}",
                        "█".repeat(vol_f),
                        "─".repeat(vol_w - vol_f.min(vol_w))
                    ),
                    Style::default().fg(Color::Green),
                ),
            ])])
            .alignment(Alignment::Right),
            cols[2],
        );
    }
}

fn block_for(title: &str, focused: bool) -> Block<'_> {
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

fn inset(area: Rect) -> Rect {
    Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

pub fn fmt_duration(secs: Option<i32>) -> String {
    match secs {
        None => "--:--".to_string(),
        Some(s) => format!("{}:{:02}", s / 60, s % 60),
    }
}

fn fmt_duration_i32(secs: i32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}
