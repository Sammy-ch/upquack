mod app;
mod ui;
mod utils;

use crate::app::App;
use std::io;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app_init = App::new();
    let run_upquack = app_init.run(&mut terminal);
    ratatui::restore();
    run_upquack
}
