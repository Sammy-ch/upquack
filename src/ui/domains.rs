use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug)]
pub struct DomainList;

impl Widget for &DomainList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let header = Line::from("URL Monitoring").left_aligned();

        Block::bordered().title_top(header).render(area, buf);
    }
}
