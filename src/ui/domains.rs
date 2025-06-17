use std::io;

use crate::ui::popup::{self, Popup};
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

#[derive(Debug, Default)]
pub struct DomainScreen {
    state: TableState,
    items: Vec<u8>,
    show_popup: bool,
    // scroll_state: ScrollbarState,
    // colors: TableColors,
    // color_index: usize,
}

impl DomainScreen {
    pub fn init() -> Self {
        DomainScreen {
            state: TableState::new(),
            show_popup: false,
            items: vec![1, 2, 3],
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Char('A') => {
                if !self.show_popup {
                    self.show_popup = true
                };
                true
            }
            KeyCode::Char('R') => true,
            KeyCode::Esc => {
                if self.show_popup {
                    self.show_popup = false
                }
                true
            }
            _ => false,
        }
    }
}

impl Widget for &DomainScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let instructions = Line::from(vec![
            "A: Add ".into(),
            "D: Delete ".into(),
            "R: Refresh ".into(),
            "Q: Quit ".into(),
        ]);
        let header = Line::from("URL Monitoring").left_aligned();

        Block::bordered()
            .title_top(header)
            .title_bottom(instructions.centered())
            .render(area, buf);

        if self.show_popup {
            let popup_block = Block::bordered().title("Add New Domain");
            let popup_area = Popup::centered_rect(60, 20, area);

            // Placeholder for popup content
            let popup_content = Paragraph::new("Enter URL: \n\n (Press Esc to close)");
            popup_content.render(popup_area, buf);
            popup_block.render(popup_area, buf);
        }
    }
}
