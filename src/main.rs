use color_eyre::eyre::Result;
use tokio::sync::mpsc;

use crate::{
    core::{
        app::App,
        event::{AppEvent, UiCmd},
    },
    ui::runtime,
};
mod core;
mod services;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let (cmd_tx, cmd_rx) = mpsc::channel::<UiCmd>(32);
    let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(32);

    let app_task = tokio::spawn(async move { App::run(event_tx, cmd_rx).await });
    let mut term = ratatui::init();
    let ui_res = runtime::run(&mut term, cmd_tx, &mut event_rx).await;
    ratatui::restore();

    if app_task.is_finished() {
        app_task.await??;
    } else {
        app_task.abort();
    }

    Ok(())
}
