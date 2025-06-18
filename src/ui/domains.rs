use crate::ui::popup::Popup; // Assuming your Popup struct is in ui/popup.rs
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Clear, ScrollbarState, TableState}; // Added Clear for popup background
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use tui_textarea::{Input, Key};

// --- Data Structures ---
#[derive(Debug, Clone)] // Added Clone trait for easier data manipulation
struct Data {
    url: String,
    status: DomainStatus,
    last_check: String,
    response_time: String,
    http_code: HttpCode,
    interval: String,
}

#[derive(Debug, Clone)]
enum DomainStatus {
    UP,
    DOWN,
    WARNING,
}

#[derive(Debug, Clone)]
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

// --- DomainScreen (previously DomainTable) ---
#[derive(Debug, Default)] // Removed `Default` because we manually initialize the popup field
pub struct DomainScreen {
    state: TableState,
    items: Vec<Data>, // Changed from Vec<u8> to Vec<Data>
    show_popup: bool,
    add_domain_popup: Option<Popup<'static>>, // Holds the Popup instance
}

impl DomainScreen {
    pub fn init() -> Self {
        let mut screen = DomainScreen {
            state: TableState::new(),
            show_popup: false,
            add_domain_popup: None, // No popup initially
            items: vec![
                // Dummy data
                Data {
                    url: "https://google.com".to_string(),
                    status: DomainStatus::UP,
                    last_check: "2025-06-18 10:00".to_string(),
                    response_time: "50ms".to_string(),
                    http_code: HttpCode::OK,
                    interval: "60s".to_string(),
                },
                Data {
                    url: "https://example.com/broken".to_string(),
                    status: DomainStatus::DOWN,
                    last_check: "2025-06-18 10:01".to_string(),
                    response_time: "Timeout".to_string(),
                    http_code: HttpCode::ERR,
                    interval: "30s".to_string(),
                },
                Data {
                    url: "https://warning.net".to_string(),
                    status: DomainStatus::WARNING,
                    last_check: "2025-06-18 10:02".to_string(),
                    response_time: "1200ms".to_string(),
                    http_code: HttpCode::OK,
                    interval: "120s".to_string(),
                },
            ],
        };
        screen.state.select(Some(0)); // Select the first item by default
        screen
    }

    /// Handles incoming keyboard events.
    /// Returns `true` if the event was consumed, `false` otherwise.
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        if self.show_popup {
            // If the popup is active, delegate key events to it
            if let Some(popup) = &mut self.add_domain_popup {
                // First, handle keys that should close or submit the popup
                match key_event.code {
                    KeyCode::Esc => {
                        self.show_popup = false;
                        self.add_domain_popup = None; // Clean up the popup instance
                        return true; // Event consumed
                    }
                    KeyCode::Enter => {
                        let input_url = popup.get_input_text().join("\n"); // Get all lines as a single string
                        if !input_url.trim().is_empty() {
                            self.add_new_domain(input_url);
                        }
                        self.show_popup = false;
                        self.add_domain_popup = None; // Clean up the popup instance
                        return true; // Event consumed
                    }
                    _ => {
                        // For other keys, convert to tui_textarea::Input and pass it
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
                        return true; // Event consumed by textarea
                    }
                }
            }
            // This path should ideally not be reached if show_popup is true and add_domain_popup is None
            false
        } else {
            // Handle keys for the main table view
            match key_event.code {
                KeyCode::Char('A') | KeyCode::Char('a') => {
                    self.show_popup = true;
                    // Initialize the popup when opening it, potentially pre-filling
                    self.add_domain_popup = Some(Popup::new(
                        Line::from("Add New Domain").into(),
                        Some("https://".to_string()), // Pre-fill with HTTPS
                    ));
                    true // Event consumed: opened popup
                }
                KeyCode::Char('D') | KeyCode::Char('d') => {
                    self.delete_selected_domain();
                    true
                }
                KeyCode::Char('R') | KeyCode::Char('r') => {
                    self.refresh_domains();
                    true
                }
                KeyCode::Up => {
                    self.previous_item();
                    true
                }
                KeyCode::Down => {
                    self.next_item();
                    true
                }
                _ => false, // Event not consumed by DomainScreen
            }
        }
    }

    fn add_new_domain(&mut self, url: String) {
        let new_domain = Data {
            url,
            status: DomainStatus::UP, // Default for new domain
            last_check: "N/A".to_string(),
            response_time: "N/A".to_string(),
            http_code: HttpCode::OK,
            interval: "60s".to_string(), // You could expand the popup to ask for this
        };
        self.items.push(new_domain);
        self.state.select(Some(self.items.len() - 1)); // Select the newly added item
    }

    fn delete_selected_domain(&mut self) {
        if let Some(selected_index) = self.state.selected() {
            if selected_index < self.items.len() {
                self.items.remove(selected_index);
                // Adjust selection after removal
                if self.items.is_empty() {
                    self.state.select(None);
                } else if selected_index >= self.items.len() {
                    self.state.select(Some(self.items.len() - 1));
                } else {
                    self.state.select(Some(selected_index));
                }
            }
        }
    }

    fn refresh_domains(&mut self) {
        // Placeholder for actual refresh logic.
        // For demonstration, add a dummy entry.
        let new_url = format!("https://refreshed-{}.com", self.items.len());
        self.items.push(Data {
            url: new_url,
            status: DomainStatus::UP,
            last_check: "Just now".to_string(),
            response_time: "30ms".to_string(),
            http_code: HttpCode::OK,
            interval: "60s".to_string(),
        });
        self.state.select(Some(self.items.len() - 1)); // Select the new item
    }

    fn next_item(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous_item(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

// --- Widget Implementation for DomainScreen ---
impl Widget for &DomainScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let instructions = Line::from(vec![
            "Esc: Return to Menu ".into(), // Assuming app handles Esc to go to main menu
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

        main_block.render(area, buf); // Render the main border and title/footer

        // --- Render the popup if it's active ---
        if self.show_popup {
            if let Some(popup) = &self.add_domain_popup {
                // Calculate the popup's area
                let popup_area = Popup::centered_rect(60, 20, area); // 60% width, 20% height of parent area

                // Render the popup widget
                popup.clone().render(popup_area, buf);
            }
        }
    }
}

