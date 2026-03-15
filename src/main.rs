use color_eyre::eyre::Result;

use crate::core::app::App;

mod core;
mod services;
mod ux;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let app = App::new().await?;

    Ok(())
}
