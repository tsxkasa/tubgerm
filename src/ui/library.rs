use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use submarine::data::{Child, Lyrics, Playlist};

use crate::{core::event::SongTime, services::cache::LibaryCache};

#[derive(Default, Debug, Clone)]
pub struct LibraryState {
    pub playlists: Option<Vec<Playlist>>,
    pub albums: Option<Vec<Child>>,
    pub liked_songs: Option<Vec<Child>>,
    pub cache: Arc<RwLock<LibaryCache>>,

    pub now_playing: Option<Box<Child>>,
    pub lyrics: Option<Lyrics>,
    pub queue: VecDeque<Child>,
    pub recently_finished_queue: VecDeque<Child>,
    pub related_tracks: Vec<Child>,
    pub progress: SongTime,
    pub volume: f64,
    pub playing: bool,
}
