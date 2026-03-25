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
    PlaylistsLoaded(Vec<Playlist>),
    PlaylistTracksLoaded(String, Vec<Child>), // (title, tracks)
    AlbumsLoaded(Vec<AlbumId3>),
    ArtistsLoaded(Vec<ArtistId3>),
    AlbumTracksLoaded(String, Vec<Child>), // (album name, tracks)
    NowPlaying(Child),
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
