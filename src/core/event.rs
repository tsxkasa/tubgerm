use std::time::Duration;

use submarine::data::{AlbumId3, AlbumWithSongsId3, Child, Playlist, PlaylistWithSongs};

#[derive(Debug)]
pub enum Event {
    Crossterm(crossterm::event::Event),
    App(AppEvent),
    Tick(Duration),
}

#[derive(Default, Debug)]
pub struct SongTime {
    pub current: f64,
    pub end: f64,
}

#[derive(Debug)]
pub enum AppEvent {
    NeedsLogin { server: String, username: String },
    LoginError(String),
    Ready,
    PlaylistsLoaded(Vec<Playlist>),
    PlaylistTracksLoaded(Box<PlaylistWithSongs>),
    AlbumsLoaded(Vec<Child>),
    AlbumTracksLoaded(Box<AlbumWithSongsId3>),
    LikedSongsLoaded(Vec<Child>),
    NowPlaying(Box<Child>),
    ProgressNow(SongTime), // progress 0.0 to 1.0 out of the entire song
    PlaybackStopped,
    PlaybackResumed,
    Notify(String, NotifLevel),
    Error(String),
}

#[derive(Debug)]
pub enum NotifLevel {
    Info,
    Warning,
    Error,
    Debug,
    Trace,
}

#[derive(Debug)]
pub enum UiCmd {
    SubmitLogin {
        url: String,
        uname: String,
        password: String,
    },
    PlayTrack(String),
    StopTrack,
    FetchPlaylists,
    FetchPlaylist(String),
    FetchAlbums,
    FetchAlbum(String),
    FetchLikedSongs,
    Next,
    Prev,
    Pause,
    Resume,
    SetVolume(f64),
    Logout,
    Exit,
}
