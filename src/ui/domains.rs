use crate::ui::popup::Popup;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Clear, ScrollbarState, TableState};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use serde::{Deserialize, Serialize};
use tui_textarea::{Input, Key};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MonitoredDomain {
    url: String,
    status: DomainStatus,
    last_check: String,
    response_time: String,
    http_code: HttpCode,
    interval: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum DomainStatus {
    UP,
    DOWN,
    UNKNOWN,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug)]
enum DomainScreenMode {
    Table,
    AddDomain(Popup<'static>),
}

#[derive(Debug)]
pub struct DomainScreen {
    state: TableState,
    domains: Vec<MonitoredDomain>,
    mode: DomainScreenMode,
}

impl DomainScreen {
    pub fn init() -> Self {
        DomainScreen {
            state: TableState::new(),
            mode: DomainScreenMode::Table,
            domains: vec![
                // Dummy data
                MonitoredDomain {
                    url: "https://google.com".to_string(),
                    status: DomainStatus::UP,
                    last_check: "2025-06-18 10:00".to_string(),
                    response_time: "50ms".to_string(),
                    http_code: HttpCode::OK,
                    interval: "60s".to_string(),
                },
                MonitoredDomain {
                    url: "https://example.com/broken".to_string(),
                    status: DomainStatus::DOWN,
                    last_check: "2025-06-18 10:01".to_string(),
                    response_time: "Timeout".to_string(),
                    http_code: HttpCode::ERR,
                    interval: "30s".to_string(),
                },
                MonitoredDomain {
                    url: "https://warning.net".to_string(),
                    status: DomainStatus::UNKNOWN,
                    last_check: "2025-06-18 10:02".to_string(),
                    response_time: "1200ms".to_string(),
                    http_code: HttpCode::OK,
                    interval: "120s".to_string(),
                },
            ],
        }
    }
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        match &mut self.mode {
            DomainScreenMode::AddDomain(popup) => {
                // If the popup is active, delegate key events to it
                match key_event.code {
                    KeyCode::Esc => {
                        self.mode = DomainScreenMode::Table; // Switch back to Table mode
                        true // Event consumed by popup to close
                    }
                    KeyCode::Enter => {
                        let input_url = popup.get_input_text().join("\n");
                        // TODO: Add logic to process input_url and add to domains
                        self.mode = DomainScreenMode::Table; // Switch back to Table mode
                        true // Event consumed by popup to submit
                    }
                    _ => {
                        let tui_input = match key_event.code {
                            KeyCode::Char(c) => Input {
                                key: Key::Char(c),
                                ctrl: key_event.modifiers.contains(KeyModifiers::CONTROL),
                                alt: key_event.modifiers.contains(KeyModifiers::ALT),
                                shift: key_event.modifiers.contains(KeyModifiers::SHIFT),
                            },
                            KeyCode::Backspace => Input {
                                key: Key::Backspace,
                                ctrl: false,
                                alt: false,
                                shift: false,
                            },
                            KeyCode::Delete => Input {
                                key: Key::Delete,
                                ctrl: false,
                                alt: false,
                                shift: false,
                            },
                            KeyCode::Left => Input {
                                key: Key::Left,
                                ctrl: false,
                                alt: false,
                                shift: false,
                            },
                            KeyCode::Right => Input {
                                key: Key::Right,
                                ctrl: false,
                                alt: false,
                                shift: false,
                            },
                            KeyCode::Up => Input {
                                key: Key::Up,
                                ctrl: false,
                                alt: false,
                                shift: false,
                            },
                            KeyCode::Down => Input {
                                key: Key::Down,
                                ctrl: false,
                                alt: false,
                                shift: false,
                            },
                            KeyCode::Tab => Input {
                                key: Key::Tab,
                                ctrl: false,
                                alt: false,
                                shift: false,
                            },
                            KeyCode::Home => Input {
                                key: Key::Home,
                                ctrl: false,
                                alt: false,
                                shift: false,
                            },
                            KeyCode::End => Input {
                                key: Key::End,
                                ctrl: false,
                                alt: false,
                                shift: false,
                            },
                            KeyCode::PageUp => Input {
                                key: Key::PageUp,
                                ctrl: false,
                                alt: false,
                                shift: false,
                            },
                            KeyCode::PageDown => Input {
                                key: Key::PageDown,
                                ctrl: false,
                                alt: false,
                                shift: false,
                            },
                            _ => return false, // If tui-textarea doesn't support it, don't consume
                        };
                        popup.textarea_mut().input(tui_input);
                        true // Event consumed by textarea
                    }
                }
            }
            DomainScreenMode::Table => {
                // Handle keys for the main table view
                match key_event.code {
                    KeyCode::Char('A') | KeyCode::Char('a') => {
                        // Switch to AddDomain mode and create the popup
                        self.mode = DomainScreenMode::AddDomain(Popup::new(
                            Line::from("Add New Domain"),
                            Some("https://".to_string()),
                        ));
                        true // Event consumed
                    }
                    KeyCode::Char('D') | KeyCode::Char('d') => {
                        // TODO: Implement delete logic
                        true // Event consumed
                    }
                    KeyCode::Char('R') | KeyCode::Char('r') => {
                        // TODO: Implement refresh logic
                        true // Event consumed
                    }
                    KeyCode::Up => {
                        // TODO: Implement table navigation
                        true // Event consumed
                    }
                    KeyCode::Down => {
                        // TODO: Implement table navigation
                        //
                        true // Event consumed
                    }
                    // return false so the parent `App` can potentially handle it.
                    KeyCode::Esc => false, // Let App handle global Esc
                    _ => false,            // Event not consumed by DomainScreen (in Table mode)
                }
            }
        }
    }
}

impl Widget for &DomainScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let instructions = Line::from(vec![
            "Esc: Return to Menu ".into(),
            "A: Add ".into(),
            "D: Delete ".into(),
            "R: Refresh ".into(),
            "Q: Quit ".into(),
            "Up/Down: Navigate ".into(),
        ]);
        let header = Line::from("URL Monitoring").left_aligned();

        let main_block = Block::bordered()
            .title_top(header)
            .title_bottom(instructions.centered());

        main_block.render(area, buf);

        if let DomainScreenMode::AddDomain(popup) = &self.mode {
            let popup_area = Popup::centered_rect(60, 20, area);
            Clear.render(popup_area, buf);
            popup.clone().render(popup_area, buf);
        }
        // Render the actual table domain table when in table mode
    }
}
