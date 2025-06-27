use std::{fs, io};

use crate::ui::domain_table::{DomainTable, DomainTableState};

use crate::ui::history_screen::{HistoryScreen, HistoryTableState};
use crate::ui::popup::Popup;
use crate::utils::is_valid_url;
use chrono::{DateTime, Utc};
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

static FILE_PATH: &str = "db/domains.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredDomain {
    pub id: Uuid,
    pub url: String,
    pub interval_seconds: u64,
    pub check_history: Vec<CheckResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub timestamp: DateTime<Utc>,
    pub status: DomainStatus,
    pub http_code: Option<HttpCode>,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainStatus {
    UP,
    DOWN,
    UNKNOWN,
    Error(String),
}

#[repr(u16)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpCode {
    OK = 200,
    ERR = 500,
    Other(u16),
    Timeout,
    NetworkError,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum DomainScreenMode {
    Table,
    AddDomain(Popup<'static>),
    DomainHistory(HistoryScreen),
}

#[derive(Debug)]
pub struct DomainScreen {
    pub domain_table_state: DomainTableState,
    pub history_table_state: HistoryTableState,
    domains: Vec<MonitoredDomain>,
    mode: DomainScreenMode,
}

impl DomainScreen {
    pub fn init() -> Self {
        let domains = Self::load_domains(FILE_PATH).unwrap_or_default();

        DomainScreen {
            domain_table_state: DomainTableState::default(),
            history_table_state: HistoryTableState::default(),
            mode: DomainScreenMode::Table,
            domains,
        }
    }

    fn save_domains(domains: &[MonitoredDomain], file_path: &str) -> io::Result<()> {
        let domain_data = serde_json::to_string_pretty(domains)?;

        fs::write(file_path, domain_data)?;
        Ok(())
    }

    fn load_domains(file_path: &str) -> io::Result<Vec<MonitoredDomain>> {
        let domain_data = fs::read_to_string(file_path)?;
        let domains: Vec<MonitoredDomain> = serde_json::from_str(&domain_data)?;
        Ok(domains)
    }

    fn delete_entry(&mut self) {
        if let Some(selected_index) = self.domain_table_state.table_state.selected() {
            if selected_index < self.domains.len() {
                let entry_id = self.domains[selected_index].id;
                self.domains.retain(|domain| domain.id != entry_id);

                if self.domains.is_empty() {
                    self.domain_table_state.table_state.select(None);
                } else if selected_index >= self.domains.len() {
                    self.domain_table_state
                        .table_state
                        .select(Some(self.domains.len() - 1))
                } else {
                    self.domain_table_state
                        .table_state
                        .select(Some(selected_index));
                }

                if let Err(e) = Self::save_domains(&self.domains, FILE_PATH) {
                    eprintln!("Error updating domains after deletion: {}", e);
                }
            }
        }
    }

    fn next_row(&mut self) {
        let i = match self.domain_table_state.table_state.selected() {
            Some(i) => {
                if i >= self.domains.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.domain_table_state.table_state.select(Some(i));
    }

    fn previous_row(&mut self) {
        let i = match self.domain_table_state.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.domains.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.domain_table_state.table_state.select(Some(i));
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        match &mut self.mode {
            DomainScreenMode::AddDomain(popup) => {
                match key_event.code {
                    KeyCode::Esc => {
                        self.mode = DomainScreenMode::Table;
                        true
                    }
                    KeyCode::Enter => {
                        let input_url = popup.get_input_text().join("\n");

                        if !is_valid_url(&input_url) {
                            popup.set_title(Line::from(
                                "Invalid URL! (e.g., http://example.com)".red(),
                            ));
                            return true;
                        }

                        let new_domain = MonitoredDomain {
                            id: Uuid::new_v4(),
                            url: input_url.trim().to_string(),
                            interval_seconds: 60,
                            check_history: Vec::new(),
                        };

                        self.domains.push(new_domain);
                        if let Err(e) = Self::save_domains(&self.domains, "db/domains.json") {
                            eprintln!("Error saving domains: {}", e);
                        }
                        self.mode = DomainScreenMode::Table;
                        true
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
                            KeyCode::Tab => Input {
                                key: Key::Tab,
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
                        self.mode = DomainScreenMode::AddDomain(Popup::new(
                            Line::from("Add New Domain"),
                            Some("https://".to_string()),
                        ));
                        true
                    }
                    KeyCode::Char('D') | KeyCode::Char('d') => {
                        self.delete_entry();
                        true
                    }
                    KeyCode::Char('H') | KeyCode::Char('h') => {
                        let selected_domain =
                            self.domain_table_state.table_state.selected().unwrap();
                        self.mode = DomainScreenMode::DomainHistory(HistoryScreen::new(
                            self.domains[selected_domain].clone(),
                        ));
                        true
                    }

                    KeyCode::Up | KeyCode::Char('k') => {
                        self.previous_row();
                        true
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.next_row();
                        true
                    }
                    // return false so the parent `App` can potentially handle it.
                    KeyCode::Esc => false, // Let App handle global Esc
                    _ => false,            // Event not consumed by DomainScreen (in Table mode)
                }
            }

            DomainScreenMode::DomainHistory(history_mode) => {
                history_mode.handle_key_event(key_event)
            }
        }
    }
}

impl Widget for &mut DomainScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let instructions = Line::from(vec![
            " Esc: Return to Menu - ".into(),
            "A: Add - ".into(),
            "H: History - ".into(),
            "D: Delete - ".into(),
            "R: Refresh - ".into(),
            "Q: Quit - ".into(),
            "Up/Down: Navigation ".into(),
        ]);
        let header = Line::from("URL Monitoring").left_aligned();

        let main_block = Block::bordered()
            .title_top(header)
            .title_bottom(instructions.centered())
            .magenta();

        let inner_area = main_block.inner(area);

        let domain_table_widget = DomainTable::new(&self.domains);

        main_block.render(area, buf);

        domain_table_widget.render(inner_area, buf, &mut self.domain_table_state);

        if let DomainScreenMode::AddDomain(popup) = &self.mode {
            let popup_area = Popup::centered_rect(60, 20, area);
            Clear.render(popup_area, buf);
            popup.clone().render(popup_area, buf);
        }

        if let DomainScreenMode::DomainHistory(history_mode) = &self.mode {
            Clear.render(area, buf);
            history_mode
                .clone()
                .render(area, buf, &mut self.history_table_state)
        }
    }
}
