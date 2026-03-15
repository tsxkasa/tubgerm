use color_eyre::eyre::Result;
use futures::channel::mpsc::{self, channel};

use crate::core::app::{App, AppEvent, UxCmd};

mod core;
mod services;
mod ux;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let (cmd_tx, cmd_rx) = mpsc::channel::<UxCmd>(32);
    let (event_tx, event_rx) = mpsc::channel::<AppEvent>(32);

    let app_task = tokio::spawn(async move { App::run(event_tx, cmd_rx).await });

    Ok(())
}
