mod app;
mod monitor;
mod ui;
mod utils;

use crate::app::App;
use std::io;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let (event_sender, event_receiver) = mpsc::unbounded_channel();
    let mut app_init = App::new(event_sender);
    let run_upquack = app_init.run(&mut terminal, event_receiver).await;
    ratatui::restore();
    run_upquack
}
