use color_eyre::eyre::Result;

use crate::core::app::App;

mod core;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let app = App::new().await?;

    Ok(())
}
