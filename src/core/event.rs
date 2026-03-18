use std::time::Duration;

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
    Logout,
}
