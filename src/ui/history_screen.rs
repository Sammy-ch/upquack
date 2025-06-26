use crate::ui::domains::{DomainStatus, HttpCode, MonitoredDomain};
use chrono::prelude::*;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    prelude::Modifier,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Cell, Row, StatefulWidget, Table, TableState, Widget},
};

#[derive(Debug, Default)]
pub struct HistoryTableState {
    pub table_state: TableState,
}

#[derive(Debug)]
pub struct HistoryScreen<'a> {
    domain: MonitoredDomain,
    pub history_table_state: HistoryTableState,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> HistoryScreen<'a> {
    pub fn new(domain: MonitoredDomain) -> Self {
        let mut screen = Self {
            domain,
            history_table_state: HistoryTableState::default(),
            _phantom: std::marker::PhantomData,
        };
        if !screen.domain.check_history.is_empty() {
            screen
                .history_table_state
                .table_state
                .select(Some(screen.domain.check_history.len() - 1));
        }
        screen
    }

    fn navigate_history_table(&mut self, key_event: KeyEvent) {
        if self.domain.check_history.is_empty() {
            self.history_table_state.table_state.select(None);
            return;
        }

        let i = match self.history_table_state.table_state.selected() {
            Some(i) => {
                match key_event.code {
                    KeyCode::Down => {
                        if i >= self.domain.check_history.len() - 1 {
                            0 // Wrap around to the beginning
                        } else {
                            i + 1
                        }
                    }
                    KeyCode::Up => {
                        if i == 0 {
                            self.domain.check_history.len() - 1 // Wrap around to the end
                        } else {
                            i - 1
                        }
                    }
                    _ => i, // No horizontal navigation for history
                }
            }
            None => 0, // If nothing selected, select the first item
        };
        self.history_table_state.table_state.select(Some(i));
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Esc => {
                // This indicates that the screen should be exited and go back to the previous one
                false // Return false to indicate event was not consumed by this screen (allowing App to switch)
            }
            KeyCode::Up => {
                self.navigate_history_table(key_event);
                true // Event consumed
            }
            KeyCode::Down => {
                self.navigate_history_table(key_event);
                true // Event consumed
            }
            _ => false, // Event not consumed by this screen
        }
    }
}

impl<'a> StatefulWidget for HistoryScreen<'a> {
    type State = HistoryTableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let title = format!("History for: {}", self.domain.url);
        let instructions = Line::from("Esc: Go Back | Up/Down: Navigate History");

        let main_block = Block::bordered()
            .title_top(Line::from(title).centered())
            .title_bottom(instructions.centered());

        Widget::render(&main_block, area, buf);

        let inner_area = main_block.inner(area);

        let header_cells = [
            "Timestamp",
            "Status",
            "HTTP Code",
            "Response Time",
            "Error Message",
        ]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().bold()));
        let header = Row::new(header_cells)
            .height(1)
            .bottom_margin(1)
            .style(Style::default().bg(Color::Blue).fg(Color::White));

        let rows: Vec<Row> = self
            .domain
            .check_history
            .iter()
            .enumerate()
            .map(|(i, check)| {
                let row_color = if i % 2 == 0 {
                    Color::DarkGray
                } else {
                    Color::Reset
                };

                let timestamp_display = check
                    .timestamp
                    .with_timezone(&Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();
                let status_display = match &check.status {
                    DomainStatus::UP => Span::styled("UP", Style::default().green().bold()),
                    DomainStatus::DOWN => Span::styled("DOWN", Style::default().red().bold()),
                    DomainStatus::UNKNOWN => {
                        Span::styled("UNKNOWN", Style::default().yellow().bold())
                    }
                    DomainStatus::Error(_) => Span::styled("ERROR", Style::default().red()),
                };
                let http_code_display = match &check.http_code {
                    Some(HttpCode::OK) => Span::styled("200 OK", Style::default().green()),
                    Some(HttpCode::ERR) => Span::styled("500 ERR", Style::default().red()),
                    Some(HttpCode::Other(c)) => {
                        Span::styled(format!("{}", c), Style::default().yellow())
                    }
                    Some(HttpCode::Timeout) => Span::styled("Timeout", Style::default().red()),
                    Some(HttpCode::NetworkError) => Span::styled("Net Err", Style::default().red()),
                    None => Span::styled("N/A", Style::default().gray()),
                };
                let response_time_display = check
                    .response_time_ms
                    .map(|ms| format!("{}ms", ms))
                    .unwrap_or_else(|| "N/A".to_string());
                let error_message_display =
                    check.error_message.as_deref().unwrap_or("").to_string();

                let cells = vec![
                    Cell::from(timestamp_display),
                    Cell::from(status_display),
                    Cell::from(http_code_display),
                    Cell::from(response_time_display),
                    Cell::from(error_message_display),
                ];
                Row::new(cells).style(Style::default().bg(row_color))
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(20), // Timestamp
                Constraint::Length(10), // Status
                Constraint::Length(12), // HTTP Code
                Constraint::Length(15), // Response Time
                Constraint::Min(0),     // Error Message (takes remaining space)
            ],
        )
        .header(header)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

        StatefulWidget::render(table, inner_area, buf, &mut state.table_state);
    }
}

