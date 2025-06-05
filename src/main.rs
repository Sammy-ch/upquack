use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
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

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }
    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
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

        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q>".blue().bold(),
        ]);

        let block = Block::bordered()
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        // Convert ASCII art to `Text`
        let banner_lines = upquack_title
            .trim_matches('\n')
            .lines()
            .map(|line| Line::from(line.blue()))
            .collect::<Vec<_>>();

        let text = Text::from(banner_lines);

        let paragraph = Paragraph::new(text).centered().block(block);

        paragraph.render(area, buf);
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}
