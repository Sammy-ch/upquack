mod app;
mod monitor;
mod ui;
mod utils;

use crate::app::App;
use ftail::Ftail;
use log::LevelFilter;
use std::{io, path::Path};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> io::Result<()> {
    let error_log_file = Path::new("log/error.log");
    let debug_log_file = Path::new("log/debug.log");

    Ftail::new()
        .single_file(debug_log_file, false, LevelFilter::Debug)
        .single_file(error_log_file, false, LevelFilter::Error)
        .init()
        .unwrap();

    let mut terminal = ratatui::init();
    let (event_sender, event_receiver) = mpsc::unbounded_channel();
    let mut app_init = App::new(event_sender);
    let run_upquack = app_init.run(&mut terminal, event_receiver).await;
    ratatui::restore();
    run_upquack
}
