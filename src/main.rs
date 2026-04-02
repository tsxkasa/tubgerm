use color_eyre::eyre::Result;
use tokio::sync::{mpsc, watch};

use crate::{
    core::{
        app::App,
        event::{AppEvent, UiCmd},
    },
    ui::{library::LibraryState, runtime},
};
mod core;
mod services;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let (cmd_tx, cmd_rx) = mpsc::channel::<UiCmd>(32);
    let (event_tx, event_rx) = mpsc::channel::<AppEvent>(32);
    let (library_tx, library_rx) = watch::channel(LibraryState::default());

    let app_task = tokio::spawn(async move { App::run(event_tx, cmd_rx, library_tx).await });
    let mut term = ratatui::init();
    let ui_res = runtime::run(&mut term, cmd_tx, event_rx, library_rx).await;

    ratatui::restore();
    ui_res?;

    if app_task.is_finished() {
        app_task.await??;
    } else {
        app_task.abort();
    }

    Ok(())
}
