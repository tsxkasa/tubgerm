use submarine::data::{AlbumId3, ArtistId3, Child, Playlist};

#[derive(Default, Debug)]
pub struct LibraryState {
    pub playlists: Vec<Playlist>,
    pub albums: Vec<AlbumId3>,
    pub artists: Vec<ArtistId3>,
    pub current_tracks: Vec<Child>,
    pub current_title: String,
    pub now_playing: Option<Child>,
    pub queue: Vec<Child>,
    pub related_tracks: Vec<Child>,
    pub progress: f64,
    pub playing: bool,
    pub volume: u8,
}
