use std::time::Duration;

use submarine::data::{AlbumId3, ArtistId3, Child, Playlist};

#[derive(Debug)]
pub enum Event {
    Crossterm(crossterm::event::Event),
    App(AppEvent),
    Tick(Duration),
}

#[derive(Debug)]
pub enum AppEvent {
    NeedsLogin { server: String, username: String },
    LoginError(String),
    Ready,
    PlaylistsLoaded(Box<Vec<Playlist>>),
    PlaylistTracksLoaded(String, Box<Vec<Child>>), // (title, tracks)
    AlbumsLoaded(Box<Vec<ArtistId3>>),
    ArtistsLoaded(Box<Vec<ArtistId3>>),
    AlbumTracksLoaded(String, Box<Vec<Child>>), // (album name, tracks)
    NowPlaying(Box<Child>),
    ProgressTick(f64),
    PlaybackStopped,
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
    FetchArtists,
    FetchArtist(String),
    FetchLikedSongs,
    Next,
    Prev,
    Pause,
    Resume,
    SetVolume(u8),
    Logout,
    Exit,
}
