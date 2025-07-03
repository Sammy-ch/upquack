use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::io;

use crate::ui::domains::DomainScreen;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum AppEvent {
    SwitchToDomainsScreen,
}

#[derive(Debug)]
pub struct App {
    current_screen: Menu,
    exit: bool,
    event_sender: mpsc::UnboundedSender<AppEvent>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Default)]
enum Menu {
    #[default]
    Main,
    Domains(DomainScreen),
}

impl App {
    pub fn new(event_sender: mpsc::UnboundedSender<AppEvent>) -> Self {
        App {
            current_screen: Menu::Main,
            exit: false,
            event_sender,
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        mut event_receiver: mpsc::UnboundedReceiver<AppEvent>,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            self.handle_input_events()?;

            // Process internal AppEvents (like screen transitions)
            // Use try_recv() to not block if no event is ready
            while let Ok(event) = event_receiver.try_recv() {
                match event {
                    AppEvent::SwitchToDomainsScreen => {
                        self.current_screen = Menu::Domains(DomainScreen::init().await);
                    }
                }
            }

            if self.exit {
                break;
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match &mut self.current_screen {
            Menu::Main => frame.render_widget(self, frame.area()),
            Menu::Domains(domain_screen) => frame.render_widget(domain_screen, frame.area()),
        }
    }

    fn handle_input_events(&mut self) -> io::Result<()> {
        if event::poll(tokio::time::Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    let consumed = match &mut self.current_screen {
                        Menu::Main => self.handle_global_key_event(key_event),
                        Menu::Domains(domain_screen) => domain_screen.handle_key_event(key_event),
                    };

                    if !consumed {
                        self.handle_global_key_event(key_event);
                    }
                }
                _ => {}
            };
        }
        Ok(())
    }

    fn handle_global_key_event(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.exit = true;
                true
            }
            KeyCode::Esc => {
                if let Menu::Domains(_) = self.current_screen {
                    self.current_screen = Menu::Main;
                    true
                } else {
                    false
                }
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                // Send an event to the main async loop to switch screens
                if !matches!(self.current_screen, Menu::Domains(_)) {
                    if let Err(e) = self.event_sender.send(AppEvent::SwitchToDomainsScreen) {
                        eprintln!("Error sending event: {}", e);
                    }
                    true
                } else {
                    false // Already on Domains screen, no action needed
                }
            }
            _ => false,
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let upquack_title = "
██╗   ██╗██████╗  ██████╗ ██╗    ██╗ █████╗  ██████╗██╗  ██╗
██║   ██║██╔══██╗██╔═══██╗██║    ██║██╔══██╗██╔════╝██║ ██╔╝
██║   ██║██████╔╝██║   ██║██║    ██║███████║██║     █████╔╝ 
██║   ██║██╔═══╝ ██║▄▄ ██║██║    ██║██╔══██║██║     ██╔═██╗ 
╚██████╔╝██║    ╚██████╔╝╚██████╔╝██║  ██║╚██████╗██║  ██╗
 ╚═════╝ ╚═╝     ╚══▀▀═╝  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝
";
        let instructions = Line::from(vec![
            " Quit ".into(),
            "<Q> ".blue().bold(),
            " - ".into(),
            "Manage URLs ".into(),
            "<E> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner_area = block.inner(area);

        let box_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner_area);

        let banner_lines = upquack_title
            .trim_matches('\n')
            .lines()
            .map(|line| Line::from(line.yellow()))
            .collect::<Vec<_>>();

        let text = Text::from(banner_lines);

        let menu_options = Text::from(vec![Line::from("Monitored URLs               E")])
            .style(Color::LightBlue)
            .centered();

        let header = Paragraph::new(text).centered();

        block.render(area, buf);
        header.render(box_layout[0], buf);
        menu_options.render(box_layout[1], buf);
    }
}
