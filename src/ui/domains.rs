use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{ScrollbarState, TableState};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug)]
struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    selected_column_style_fg: Color,
    selected_cell_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

#[derive(Debug)]
pub struct DomainTable {
    state: TableState,
    items: Vec<u8>,
    // scroll_state: ScrollbarState,
    // colors: TableColors,
    // color_index: usize,
}

impl DomainTable {
    pub fn init() -> Self {
        DomainTable {
            state: TableState::new(),
            items: vec![1, 2, 3],
        }
    }
}

#[derive(Debug)]
struct Data {
    url: String,
    status: DomainStatus,
    last_check: String,
    response_time: String,
    http_code: HttpCode,
    interval: String,
}

#[derive(Debug)]
enum DomainStatus {
    UP,
    DOWN,
    WARNING,
}

#[derive(Debug)]
enum HttpCode {
    OK = 200,
    ERR = 500,
}

impl Widget for &DomainTable {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let header = Line::from("URL Monitoring").left_aligned();

        Block::bordered().title_top(header).render(area, buf);
    }
}
