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
    // The TextArea itself. Renamed from `content` to `textarea` for clarity.
    textarea: TextArea<'a>,
    border_style: Style,
    style: Style,
    title_style: Style,
}

impl<'a> Clone for Popup<'a> {
    fn clone(&self) -> Self {
        // Clone the simple fields directly
        let title_clone = self.title.clone();
        let border_style_clone = self.border_style;
        let style_clone = self.style;
        let title_style_clone = self.title_style;

        // For TextArea, create a *new* instance and copy its content.
        // You cannot simply clone the TextArea directly.
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
        // Apply default block styling to the textarea itself
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
            style: Style::default().bg(Color::DarkGray), // Background for the whole popup
            title_style: Style::default()
                .fg(Color::White)
                .add_modifier(ratatui::style::Modifier::BOLD),
        }
    }

    /// Provides mutable access to the internal TextArea for event handling.
    pub fn textarea_mut(&mut self) -> &mut TextArea<'a> {
        &mut self.textarea
    }

    /// Retrieves the lines of text from the TextArea.
    pub fn get_input_text(&self) -> Vec<String> {
        self.textarea
            .lines()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Helper function to create a centered Rect for the popup.
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
        // 1. Clear the area behind the popup
        Clear.render(area, buf);

        // 2. Create the main block for the popup
        let block = Block::bordered()
            .title(self.title)
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_style(self.border_style)
            .style(self.style); // Apply the overall popup style

        // 3. Calculate the inner area for the content (the textarea)
        let inner_area = block.inner(area);

        // 4. Render the main popup block
        block.render(area, buf);

        // 5. Render the TextArea content within the inner area
        self.textarea.render(inner_area, buf);
    }
}
