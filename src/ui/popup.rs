use ratatui::prelude::{Constraint, Direction, Layout, Rect};
use ratatui::{
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Widget},
};
use tui_textarea::TextArea;

#[derive(Debug)]
pub struct Popup<'a> {
    title: Line<'a>,
    textarea: TextArea<'a>,
    border_style: Style,
    style: Style,
    title_style: Style,
}

impl<'a> Clone for Popup<'a> {
    fn clone(&self) -> Self {
        let title_clone = self.title.clone();
        let border_style_clone = self.border_style;
        let style_clone = self.style;
        let title_style_clone = self.title_style;

        //Create a *new* instance and copy its content.
        let mut cloned_textarea = TextArea::default();
        cloned_textarea.insert_str(self.textarea.lines().join("\n")); // Copy all lines

        // Also, copy the block configuration from the original textarea to the new one
        if let Some(block) = self.textarea.block() {
            // This re-applies the border, title, and style to the cloned textarea
            cloned_textarea.set_block(block.clone());
        }

        Self {
            title: title_clone,
            textarea: cloned_textarea,
            border_style: border_style_clone,
            style: style_clone,
            title_style: title_style_clone,
        }
    }
}

impl<'a> Popup<'a> {
    pub fn new(title: Line<'a>, initial_content: Option<String>) -> Self {
        let mut textarea = TextArea::default();
        if let Some(content) = initial_content {
            textarea.insert_str(content);
        }
        textarea.set_block(
            Block::bordered()
                .borders(Borders::ALL)
                .title("Enter URL")
                .style(Style::default().fg(Color::LightCyan)),
        );

        Self {
            title,
            textarea,
            border_style: Style::default().fg(Color::Gray),
            style: Style::default().bg(Color::DarkGray),
            title_style: Style::default()
                .fg(Color::White)
                .add_modifier(ratatui::style::Modifier::BOLD),
        }
    }

    pub fn textarea_mut(&mut self) -> &mut TextArea<'a> {
        &mut self.textarea
    }

    pub fn set_title(&mut self, title: Line<'a>) {
        self.title = title;
    }

    pub fn get_input_text(&self) -> Vec<String> {
        self.textarea
            .lines()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

impl<'a> Widget for Popup<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Clear.render(area, buf);

        let block = Block::bordered()
            .title(self.title)
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_style(self.border_style)
            .style(self.style);

        let inner_area = block.inner(area);

        block.render(area, buf);

        self.textarea.render(inner_area, buf);
    }
}
