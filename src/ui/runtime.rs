use color_eyre::eyre::Result;
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

pub async fn run(
    term: &mut DefaultTerminal,
    cmd_tx: Sender<UiCmd>,
    event_rx: Receiver<AppEvent>,
) -> Result<()> {
    let mut ui = Ui::new(event_rx, cmd_tx);
    let mut crossterm_events = EventStream::new();
    let mut tick = interval(TICK_RATE);

    loop {
        term.draw(|f| ui.render(f))?;

        let event = tokio::select! {
           ct_event_opt = crossterm_events.next() => {
                match ct_event_opt {
                    Some(Ok(ct_event)) => Event::Crossterm(ct_event),
                    _ => break,
                }
            }
            app_event_opt = ui.event_rx().recv() => {
                match app_event_opt {
                    Some(app_event) => Event::App(app_event),
                    None => break,
                }
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
