use std::{collections::HashMap, ops::Index};

use submarine::data::{
    AlbumId3, AlbumWithSongsId3, ArtistId3, ArtistWithAlbumsId3, Child, IndexId3, Playlist,
    PlaylistWithSongs,
};

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

#[derive(Default, Debug)]
pub struct LibraryState {
    pub playlists: Option<Vec<Playlist>>,
    pub albums: Option<Vec<Child>>,
    pub liked_songs: Option<Vec<Child>>,
    pub playlist_cache: HashMap<String, PlaylistWithSongs>,
    pub album_cache: HashMap<String, AlbumWithSongsId3>,
    pub liked_cache: HashMap<String, Child>,

    pub now_playing: Option<Box<Child>>,
    pub queue: Vec<Child>,
    pub related_tracks: Vec<Child>,
    pub progress: f64,
    pub playing: bool,
    pub volume: u8,
}
