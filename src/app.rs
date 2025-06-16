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
    domain_screen: DomainScreen,
    exit: bool,
}

#[derive(Debug, Default)]
enum Menu {
    #[default]
    Main,
    Domains,
    AlertActions,
    HistoricalData,
}

impl App {
    pub fn new() -> Self {
        App {
            current_screen: Menu::Main,
            exit: false,
            domain_screen: DomainScreen::init(),
        }
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        match self.current_screen {
            Menu::Main => frame.render_widget(self, frame.area()),
            Menu::Domains => frame.render_widget(&self.domain_screen, frame.area()),
            Menu::AlertActions => frame.render_widget(Paragraph::new("AlertActions"), frame.area()),
            Menu::HistoricalData => {
                frame.render_widget(Paragraph::new("HistoricalData"), frame.area())
            }
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match self.current_screen {
                    Menu::Main => self.handle_global_key_event(key_event),
                    Menu::Domains => {
                        self.domain_screen.handle_key_event(key_event);

                        if !self.domain_screen.handle_key_event(key_event) {
                            self.handle_global_key_event(key_event);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_global_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc => self.current_screen = Menu::Main,
            KeyCode::Char('e') => self.current_screen = Menu::Domains,
            KeyCode::Char('m') => self.current_screen = Menu::AlertActions,
            KeyCode::Char('p') => self.current_screen = Menu::HistoricalData,
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
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

        let menu_options = Text::from(vec![
            Line::from("Monitored URLS                 E"),
            Line::from("Alert Actions                  M"),
            Line::from("Historical Data                P"),
        ])
        .style(Color::LightBlue)
        .centered();

        let header = Paragraph::new(text).centered();

        block.render(area, buf);
        header.render(box_layout[0], buf);
        menu_options.render(box_layout[1], buf);
    }
}
