use crate::config::Config;
use crate::ui::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Clear, List, ListItem, Paragraph},
    Frame,
};

/// A popup widget for displaying the current configuration.
#[derive(Debug)]
pub struct ConfigPopup {
    /// Whether the popup is visible
    visible: bool,
}

impl ConfigPopup {
    /// Creates a new [`ConfigPopup`].
    pub fn new() -> Self {
        Self {
            visible: false,
        }
    }

    /// Toggles the visibility of the popup.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Returns whether the popup is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Renders the popup.
    pub fn render(&mut self, frame: &mut Frame, area: ratatui::layout::Rect, theme: &Theme, config: &Config) {
        if !self.visible {
            return;
        }

        // Create a centered popup
        let popup_area = centered_rect(60, 40, area);

        // Create a block with double borders for more distinction
        let block = Block::default()
            .title(" Configuration ")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .style(theme.main);

        // Create a layout for the popup
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Description
                Constraint::Min(10),    // Content area
            ])
            .margin(2)
            .split(popup_area);

        // Render the block and clear the area
        frame.render_widget(Clear, popup_area);
        frame.render_widget(block, popup_area);

        // Render description
        let description = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("Current ArXiv search preferences. Press "),
                Span::styled("c", theme.highlight),
                Span::raw(" to close."),
            ]),
        ])
        .style(theme.main);
        frame.render_widget(description, layout[0]);

        // Create list items for current configuration
        let items = vec![
            ListItem::new(vec![Line::from(vec![
                Span::raw("Category: "),
                Span::styled(&config.query.category, theme.highlight),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::raw("Authors: "),
                Span::styled(
                    config.highlight.authors.as_ref().map_or("None".to_string(), |a| a.join(", ")),
                    theme.highlight,
                ),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::raw("Keywords: "),
                Span::styled(
                    config.highlight.keywords.as_ref().map_or("None".to_string(), |k| k.join(", ")),
                    theme.highlight,
                ),
            ])]),
        ];

        // Create and render the list
        let list = List::new(items)
            .block(Block::default().borders(Borders::NONE))
            .style(theme.main);

        frame.render_widget(list, layout[1]);
    }
}

/// Helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
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