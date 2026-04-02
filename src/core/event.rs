use std::time::Duration;

#[derive(Debug)]
pub enum Event {
    Crossterm(crossterm::event::Event),
    App(AppEvent),
    Tick(Duration),
}

#[derive(Default, Debug, Clone)]
pub struct SongTime {
    pub current: f64,
    pub end: f64,
}

#[derive(Debug)]
pub enum AppEvent {
    NeedsLogin { server: String, username: String },
    LoginError(String),
    Ready,
    // LibraryUpdated(LibraryState),
    // PlaylistsLoaded(Vec<Playlist>),
    // PlaylistTracksLoaded(Box<PlaylistWithSongs>),
    // AlbumsLoaded(Vec<Child>),
    // AlbumTracksLoaded(Box<AlbumWithSongsId3>),
    // LikedSongsLoaded(Vec<Child>),
    // NowPlaying(Box<Child>),
    // QueueGenerated(VecDeque<Child>),
    // VolumeChanged(f64),
    // ProgressNow(SongTime),
    // PlaybackStopped,
    // PlaybackResumed,
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
pub enum PlayFrom {
    LikedSongs,
    Playlist(String),
    Album(String),
}

#[derive(Debug)]
pub enum UiCmd {
    SubmitLogin {
        url: String,
        uname: String,
        password: String,
    },
    PlayTrack(String, PlayFrom),
    StopTrack,
    FetchPlaylists,
    FetchPlaylist(String),
    FetchAlbums,
    FetchAlbum(String),
    FetchLyrics(String),
    FetchLikedSongs,
    Next,
    Prev,
    Pause,
    Resume,
    SetVolume(f64),
    Logout,
    Exit,
}
