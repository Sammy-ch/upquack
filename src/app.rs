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

#[derive(Debug, Default)]
pub struct App {
    current_screen: Menu,
    exit: bool,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Default)]
enum Menu {
    #[default]
    Main,
    Domains(DomainScreen),
}

impl App {
    pub async fn new() -> Self {
        let domain_screen = DomainScreen::init().await;
        App {
            current_screen: Menu::Main,
            exit: false,
        }
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match &mut self.current_screen {
            Menu::Main => frame.render_widget(self, frame.area()),
            Menu::Domains(domain_screen) => frame.render_widget(domain_screen, frame.area()),
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
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
        Ok(())
    }

    fn handle_global_key_event(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Char('q') => {
                self.exit();
                true
            }
            KeyCode::Esc => {
                self.current_screen = Menu::Main;
                true
            }
            KeyCode::Char('e') => {
                self.current_screen = Menu::Domains(DomainScreen::init());
                true
            }
            _ => false,
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let upquack_title = "
██╗   ██╗██████╗  ██████╗ ██╗   ██╗ █████╗  ██████╗██╗  ██╗
██║   ██║██╔══██╗██╔═══██╗██║   ██║██╔══██╗██╔════╝██║ ██╔╝
██║   ██║██████╔╝██║   ██║██║   ██║███████║██║     █████╔╝ 
██║   ██║██╔═══╝ ██║▄▄ ██║██║   ██║██╔══██║██║     ██╔═██╗ 
╚██████╔╝██║     ╚██████╔╝╚██████╔╝██║  ██║╚██████╗██║  ██╗
 ╚═════╝ ╚═╝      ╚══▀▀═╝  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝
";
        let instructions = Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]);

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

        let menu_options = Text::from(vec![Line::from("Monitored URLS                 E")])
            .style(Color::LightBlue)
            .centered();

        let header = Paragraph::new(text).centered();

        block.render(area, buf);
        header.render(box_layout[0], buf);
        menu_options.render(box_layout[1], buf);
    }
}
