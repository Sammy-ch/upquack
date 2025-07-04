use log::error;
use std::sync::{Arc, Mutex};

use std::{fs, io};

use crate::monitor::start_monitoring_task;
use crate::ui::domain_table::{DomainTable, DomainTableState};

use crate::ui::history_table::{HistoryTable, HistoryTableState};
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
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tui_textarea::{Input, Key};
use uuid::Uuid;

static FILE_PATH: &str = "db/domains.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredDomain {
    pub id: Uuid,
    pub url: String,
    pub interval_seconds: u64,
    pub check_history: Vec<CheckStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckStatus {
    pub timestamp: DateTime<Utc>,
    pub status: DomainStatus,
    pub http_code: Option<HttpCode>,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainStatus {
    Up,
    Down,
    Unknown,
    Error(String),
}

#[repr(u16)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpCode {
    Ok = 200,
    Err = 500,
    Other(u16),
    Timeout,
    NetworkError,
}

impl HttpCode {
    pub fn from_status_code(code: StatusCode) -> Self {
        match code {
            StatusCode::OK => HttpCode::Ok,
            _ if code.is_server_error() => HttpCode::Err,
            _ => HttpCode::Other(code.as_u16()),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum DomainScreenMode {
    DomainTable,
    AddDomain(Popup<'static>),
    HistoryTable,
}

#[derive(Debug)]
pub struct DomainScreen {
    pub domain_table_state: DomainTableState,
    pub history_table_state: HistoryTableState,
    domains: Arc<Mutex<Vec<MonitoredDomain>>>,
    mode: DomainScreenMode,
}

impl DomainScreen {
    pub async fn init() -> Self {
        let domains = Self::load_domains(FILE_PATH).unwrap_or_default();
        let domains_arc = Arc::new(Mutex::new(domains));

        let update_domains_callback = {
            let domains_arc_for_callback = Arc::clone(&domains_arc);
            Arc::new(
                move |updated_domain: &MonitoredDomain, check_history: &[CheckStatus]| {
                    let mut domains_guard = domains_arc_for_callback.lock().unwrap();
                    if let Some(d) = domains_guard.iter_mut().find(|d| d.id == updated_domain.id) {
                        d.check_history = check_history.to_vec();

                        if let Err(e) = Self::save_domains(&domains_guard, FILE_PATH) {
                            error!("Failed to save domains after check: {}", e);
                            return Err(e); // Propagate the error
                        }
                    }
                    Ok(())
                },
            )
        };

        start_monitoring_task(Arc::clone(&domains_arc), update_domains_callback).await;

        DomainScreen {
            domain_table_state: DomainTableState::default(),
            history_table_state: HistoryTableState::default(),
            mode: DomainScreenMode::DomainTable,
            domains: domains_arc,
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
        let mut domain_guard = self.domains.lock().unwrap().clone();

        if let Some(selected_index) = self.domain_table_state.table_state.selected() {
            if selected_index < domain_guard.len() {
                let entry_id = domain_guard[selected_index].id;
                domain_guard.retain(|domain| domain.id != entry_id);

                if domain_guard.is_empty() {
                    self.domain_table_state.table_state.select(None);
                } else if selected_index >= domain_guard.len() {
                    self.domain_table_state
                        .table_state
                        .select(Some(domain_guard.len() - 1))
                } else {
                    self.domain_table_state
                        .table_state
                        .select(Some(selected_index));
                }

                if let Err(e) = Self::save_domains(&domain_guard, FILE_PATH) {
                    eprintln!("Error updating domains after deletion: {}", e);
                }
            }
        }
    }

    fn next_row(&mut self) {
        let domain_guard = self.domains.lock().unwrap().clone();

        let i = match self.domain_table_state.table_state.selected() {
            Some(i) => {
                if i >= domain_guard.len() - 1 {
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
        let domain_guard = self.domains.lock().unwrap().clone();
        let i = match self.domain_table_state.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    domain_guard.len() - 1
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
            DomainScreenMode::AddDomain(popup) => match key_event.code {
                KeyCode::Esc => {
                    self.mode = DomainScreenMode::DomainTable;
                    true
                }
                KeyCode::Enter => {
                    let input_url = popup.get_input_text().join("\n");

                    if !is_valid_url(&input_url) {
                        popup
                            .set_title(Line::from("Invalid URL! (e.g., http://example.com)".red()));
                        return true;
                    }

                    let new_domain = MonitoredDomain {
                        id: Uuid::new_v4(),
                        url: input_url.trim().to_string(),
                        interval_seconds: 60,
                        check_history: Vec::new(),
                    };

                    {
                        let mut domain_guard = self.domains.lock().unwrap();
                        domain_guard.push(new_domain);
                        if let Err(e) = Self::save_domains(&domain_guard, "db/domains.json") {
                            eprintln!("Error saving domains: {}", e);
                        }
                    }

                    self.mode = DomainScreenMode::DomainTable;
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

                        _ => return false,
                    };
                    popup.textarea_mut().input(tui_input);
                    true
                }
            },
            DomainScreenMode::DomainTable => {
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
                        self.mode = DomainScreenMode::HistoryTable;
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
            DomainScreenMode::HistoryTable => {
                if let Some(selected_domain) = self.domain_table_state.table_state.selected() {
                    let domains_guard = self.domains.lock().unwrap().clone();
                    let domain_history = domains_guard[selected_domain].check_history.clone();
                    match key_event.code {
                        KeyCode::Esc => {
                            self.mode = DomainScreenMode::DomainTable;
                            true
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            HistoryTable::previous_row(
                                &mut self.history_table_state,
                                domain_history.len(),
                            );
                            true
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            HistoryTable::next_row(
                                &mut self.history_table_state,
                                domain_history.len(),
                            );
                            true
                        }

                        _ => false,
                    }
                } else {
                    false
                }
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

        let domains_guard = self.domains.lock().unwrap().clone();
        let domain_table_widget = DomainTable::new(&domains_guard);

        main_block.render(area, buf);

        domain_table_widget.render(inner_area, buf, &mut self.domain_table_state);
        drop(domains_guard);

        if let DomainScreenMode::AddDomain(popup) = &self.mode {
            let popup_area = Popup::centered_rect(60, 20, area);
            Clear.render(popup_area, buf);
            popup.clone().render(popup_area, buf);
        }

        if let DomainScreenMode::HistoryTable = &self.mode {
            Clear.render(area, buf);

            let selected_domain_index = self.domain_table_state.table_state.selected().unwrap();
            let domains = self.domains.lock().unwrap().clone();

            let history_table_widget = HistoryTable::new(domains[selected_domain_index].clone());

            history_table_widget.render(area, buf, &mut self.history_table_state);
        }
    }
}
