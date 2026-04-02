use std::{collections::VecDeque, sync::Arc};

use submarine::data::{Child, Playlist};

use crate::{core::event::SongTime, services::cache::LibaryCache};

#[derive(Default, Debug)]
pub enum MainView {
    #[default]
    Albums,
    Artists,
    Playlists,
    Playlist(String),
    Album(String),
    LikedSongs,
}

#[derive(Default, Debug, Clone)]
pub struct LibraryState {
    pub playlists: Option<Vec<Playlist>>,
    pub albums: Option<Vec<Child>>,
    pub liked_songs: Option<Vec<Child>>,
    pub cache: Arc<LibaryCache>,

    pub now_playing: Option<Box<Child>>,
    pub queue: VecDeque<Child>,
    pub recently_finished_queue: VecDeque<Child>,
    pub related_tracks: Vec<Child>,
    pub progress: SongTime,
    pub volume: f64,
    pub playing: bool,
}
