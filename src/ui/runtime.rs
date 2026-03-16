use color_eyre::eyre::{Error, Result};
use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::DefaultTerminal;
use std::time::Duration;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::interval,
};

use crate::{
    core::event::{AppEvent, Event, UiCmd},
    ui::ui::Ui,
};

const TICK_RATE: Duration = Duration::from_millis(250);

pub async fn run(cmd_tx: Sender<UiCmd>, event_rx: Receiver<AppEvent>) -> Result<()> {
    let mut term = ratatui::init();
    let result = main_run(&mut term, event_rx, cmd_tx).await;
    ratatui::restore();
    result
}

async fn main_run(
    term: &mut DefaultTerminal,
    event_rx: Receiver<AppEvent>,
    cmd_tx: Sender<UiCmd>,
) -> Result<()> {
    let mut ui = Ui::new(event_rx, cmd_tx);
    let mut crossterm_events = EventStream::new();
    let mut tick = interval(TICK_RATE);

    loop {
        term.draw(|f| ui.render(f))?;

        let event = tokio::select! {
            Some(Ok(ct_event)) = crossterm_events.next() => {
                Event::Crossterm(ct_event)
            }
            Some(app_event) = ui.event_rx().recv() => {
                Event::App(app_event)
            }
            _ = tick.tick() => {
                Event::Tick
            }
        };

        if !ui.handle_event(event).await? {
            break;
        }
    }
    Ok(())
}
