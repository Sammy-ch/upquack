use crate::ui::domains::{DomainStatus, HttpCode, MonitoredDomain};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Cell, Row, StatefulWidget, Table, TableState},
};

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

                // Handle Option<T> fields for display
                let url_display = domain.url.clone();
                let status_display = match &domain.status {
                    Some(DomainStatus::UP) => "UP".green().bold(),
                    Some(DomainStatus::DOWN) => "DOWN".red().bold(),
                    Some(DomainStatus::UNKNOWN) => "UNKNOWN".yellow().bold(),
                    Some(DomainStatus::Error(e)) => format!("Error: {}", e).red(),
                    None => "N/A".gray(),
                };
                let last_check_display = domain.last_check.as_deref().unwrap_or("N/A").to_string();
                let response_time_display =
                    domain.response_time.as_deref().unwrap_or("N/A").to_string();
                let http_code_display = match &domain.http_code {
                    Some(HttpCode::OK) => "200 OK".green(),
                    Some(HttpCode::ERR) => "500 ERR".red(),
                    None => "N/A".gray(),
                };
                let interval_display = domain.interval.as_deref().unwrap_or("N/A").to_string();

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
