use color_eyre::eyre::Result;
use futures::channel::mpsc::{self, channel};

use crate::core::app::App;

mod core;
mod services;
mod ux;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    // let (tx, rx) = mpsc::channel(10);

    // App::run(tx, rx);

    Ok(())
}
