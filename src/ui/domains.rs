use std::{fs, io};

use crate::ui::domain_table::{DomainTable, DomainTableState};
use crate::ui::popup::Popup;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::Clear;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{Block, Widget},
};
use serde::{Deserialize, Serialize};
use tui_textarea::{Input, Key};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredDomain {
    pub id: Uuid,
    pub url: String,
    pub status: Option<DomainStatus>,
    pub last_check: Option<String>,
    pub response_time: Option<String>,
    pub http_code: Option<HttpCode>,
    pub interval: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainStatus {
    UP,
    DOWN,
    UNKNOWN,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpCode {
    OK,
    ERR,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum DomainScreenMode {
    Table,
    AddDomain(Popup<'static>),
}

#[derive(Debug)]
pub struct DomainScreen {
    pub table_state: DomainTableState,
    domains: Vec<MonitoredDomain>,
    mode: DomainScreenMode,
}

impl DomainScreen {
    pub fn init() -> Self {
        let domains = match Self::load_domains("db/domains.json") {
            Ok(loaded_domains) => loaded_domains,
            Err(e) => {
                eprintln!("Could not load domains: {}", e);
                Vec::new()
            }
        };
        DomainScreen {
            table_state: DomainTableState::default(),
            mode: DomainScreenMode::Table,
            domains,
        }
    }

    pub fn save_domains(domains: &[MonitoredDomain], file_path: &str) -> io::Result<()> {
        let domain_data = serde_json::to_string_pretty(domains)?;
        fs::write(file_path, domain_data)?;
        Ok(())
    }

    pub fn load_domains(file_path: &str) -> io::Result<Vec<MonitoredDomain>> {
        let domain_data = fs::read_to_string(file_path)?;
        let domains: Vec<MonitoredDomain> = serde_json::from_str(&domain_data)?;
        Ok(domains)
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
                        if !input_url.trim().is_empty() {
                            let new_domain = MonitoredDomain {
                                id: Uuid::new_v4(),
                                url: input_url.trim().to_string(),
                                status: None,
                                http_code: None,
                                interval: None,
                                last_check: None,
                                response_time: None,
                            };

                            self.domains.push(new_domain);
                            if let Err(e) = Self::save_domains(&self.domains, "db/domains.json") {
                                eprintln!("Error saving domains: {}", e);
                            }
                        }
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

impl Widget for &mut DomainScreen {
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

        let inner_area = main_block.inner(area);

        let domain_table_widget = DomainTable::new(&self.domains);

        main_block.render(area, buf);

        domain_table_widget.render(inner_area, buf, &mut self.table_state);

        if let DomainScreenMode::AddDomain(popup) = &self.mode {
            let popup_area = Popup::centered_rect(60, 20, area);
            Clear.render(popup_area, buf);
            popup.clone().render(popup_area, buf);
        }
    }
}
