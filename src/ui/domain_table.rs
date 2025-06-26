use crate::ui::domains::{DomainStatus, HttpCode, MonitoredDomain};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Cell, Row, StatefulWidget, Table, TableState},
};

use chrono::prelude::*;

#[derive(Debug, Default)]
pub struct DomainTableState {
    pub table_state: TableState,
}

#[derive(Debug)]
pub struct DomainTable<'a> {
    domains: &'a [MonitoredDomain],
}

impl<'a> DomainTable<'a> {
    pub fn new(domains: &'a [MonitoredDomain]) -> Self {
        Self { domains }
    }
}

impl<'a> StatefulWidget for DomainTable<'a> {
    type State = DomainTableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header_cells = [
            "URL",
            "Status",
            "Last Check",
            "Response Time",
            "HTTP Code",
            "Interval",
        ]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().bold()));

        let header = Row::new(header_cells)
            .height(1)
            .bottom_margin(1)
            .style(Style::default().bg(Color::Yellow).fg(Color::Black));

        let rows: Vec<Row> = self
            .domains
            .iter()
            .enumerate()
            .map(|(i, domain)| {
                // Determine row color based on index for alternation
                let row_color = if i % 2 == 0 {
                    Color::DarkGray
                } else {
                    Color::Reset
                };

                let url_display = domain.url.clone();
                let interval_display = format!("{}s", domain.interval_seconds);

                // --- Extract the latest check result ---
                let (status_display, last_check_display, response_time_display, http_code_display) =
                    if let Some(latest_check) = domain.check_history.last() {
                        // Get the last element
                        let status = match &latest_check.status {
                            DomainStatus::UP => Span::styled("UP", Style::default().green().bold()),
                            DomainStatus::DOWN => {
                                Span::styled("DOWN", Style::default().red().bold())
                            }
                            DomainStatus::UNKNOWN => {
                                Span::styled("UNKNOWN", Style::default().yellow().bold())
                            }
                            DomainStatus::Error(e) => {
                                Span::styled(format!("Error: {}", e), Style::default().red())
                            }
                        };
                        let last_check = latest_check
                            .timestamp
                            .with_timezone(&Local)
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string();
                        let response_time = latest_check
                            .response_time_ms
                            .map(|ms| format!("{}ms", ms))
                            .unwrap_or_else(|| "N/A".to_string());
                        let http_code = match &latest_check.http_code {
                            Some(HttpCode::OK) => Span::styled("200 OK", Style::default().green()),
                            Some(HttpCode::ERR) => Span::styled("500 ERR", Style::default().red()),
                            Some(HttpCode::Other(c)) => {
                                Span::styled(format!("{}", c), Style::default().yellow())
                            }
                            Some(HttpCode::Timeout) => {
                                Span::styled("Timeout", Style::default().red())
                            }
                            Some(HttpCode::NetworkError) => {
                                Span::styled("Net Err", Style::default().red())
                            }
                            None => Span::styled("N/A", Style::default().gray()),
                        };
                        (status, last_check, response_time, http_code)
                    } else {
                        // If no check history yet
                        (
                            Span::styled("N/A", Style::default().gray()), // Status
                            "N/A".to_string(),                            // Last Check
                            "N/A".to_string(),                            // Response Time
                            Span::styled("N/A", Style::default().gray()), // HTTP Code
                        )
                    };

                let cells = vec![
                    Cell::from(url_display),
                    Cell::from(status_display),
                    Cell::from(last_check_display),
                    Cell::from(response_time_display),
                    Cell::from(http_code_display),
                    Cell::from(interval_display),
                ];
                Row::new(cells).style(Style::default().bg(row_color))
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(30), // For URL
                Constraint::Length(10),     // For Status
                Constraint::Length(18),     // For Last Check
                Constraint::Length(15),     // For Response Time
                Constraint::Length(10),     // For HTTP Code
                Constraint::Length(8),      // For Interval
            ],
        )
        .column_spacing(2)
        .header(header)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

        table.render(area, buf, &mut state.table_state);
    }
}
