#[derive(Debug)]
pub enum Event {
    Crossterm(crossterm::event::Event),
    App(AppEvent),
    Tick,
}

#[derive(Debug)]
pub enum AppEvent {
    NeedsLogin,
    LoginError(String),
    Ready,
    Error(String),
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
