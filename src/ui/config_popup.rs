use crate::config::Config;
use crate::ui::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Clear, List, ListItem, Paragraph, ListState},
    Frame,
};

/// The current mode of the config popup
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PopupMode {
    /// Viewing the current configuration
    View,
    /// Editing the configuration (planned for future implementation)
    Edit,
}

/// Errors that can occur when rendering the config popup
#[derive(Debug, thiserror::Error, Clone)]
pub enum ConfigPopupError {
    #[error("Invalid layout: {0}")]
    LayoutError(String),
    #[error("Rendering error: {0}")]
    RenderingError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// A popup widget for displaying the current configuration.
/// 
/// This widget provides a modal interface for viewing the current configuration.
/// It is designed to be extensible for future editing capabilities, with the following
/// features planned:
/// - Keyboard navigation through configuration items
/// - In-place editing of configuration values
/// - Validation of edited values
/// - Save/cancel functionality
/// 
/// Currently implemented features:
/// - Toggle visibility with keyboard shortcut
/// - Display current configuration values
/// - Error handling for layout and rendering issues
/// - Proper state management for future editing
#[derive(Debug)]
pub struct ConfigPopup {
    /// Whether the popup is visible
    visible: bool,
    /// Last error that occurred during rendering
    last_error: Option<ConfigPopupError>,
    /// Current mode of the popup (currently only View is implemented)
    mode: PopupMode,
    /// State for the list widget (prepared for future editing)
    list_state: ListState,
}

impl ConfigPopup {
    /// Creates a new [`ConfigPopup`].
    pub fn new() -> Self {
        Self {
            visible: false,
            last_error: None,
            mode: PopupMode::View,
            list_state: ListState::default(),
        }
    }

    /// Toggles the visibility of the popup.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        // Clear any previous errors when toggling visibility
        self.last_error = None;
        // Reset mode to view when toggling visibility
        self.mode = PopupMode::View;
        // Reset selection when toggling visibility
        self.list_state.select(None);
    }

    /// Returns whether the popup is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns the last error that occurred during rendering, if any.
    pub fn last_error(&self) -> Option<&ConfigPopupError> {
        self.last_error.as_ref()
    }

    /// Returns the current mode of the popup.
    pub fn mode(&self) -> PopupMode {
        self.mode
    }

    /// Returns the currently selected item index.
    pub fn selected_index(&self) -> Option<usize> {
        self.list_state.selected()
    }

    /// Renders the popup.
    pub fn render(&mut self, frame: &mut Frame, area: ratatui::layout::Rect, theme: &Theme, config: &Config) -> Result<(), ConfigPopupError> {
        if !self.visible {
            return Ok(());
        }

        // Validate area dimensions
        if area.width < 20 || area.height < 10 {
            let error = ConfigPopupError::LayoutError("Popup area too small for rendering".to_string());
            self.last_error = Some(error.clone());
            return Err(error);
        }

        // Create a centered popup
        let popup_area = centered_rect(60, 40, area);

        // Validate popup area
        if popup_area.width < 10 || popup_area.height < 5 {
            let error = ConfigPopupError::LayoutError("Calculated popup area too small".to_string());
            self.last_error = Some(error.clone());
            return Err(error);
        }

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

        // Render description based on mode
        let description = match self.mode {
            PopupMode::View => Paragraph::new(vec![
                Line::from(vec![
                    Span::raw("Current ArXiv search preferences. Press "),
                    Span::styled("c", theme.highlight),
                    Span::raw(" to close."),
                ]),
            ]),
            PopupMode::Edit => Paragraph::new(vec![
                Line::from(vec![
                    Span::raw("Editing configuration. Press "),
                    Span::styled("Esc", theme.highlight),
                    Span::raw(" to cancel, "),
                    Span::styled("Enter", theme.highlight),
                    Span::raw(" to save."),
                ]),
            ]),
        }
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
            .style(theme.main)
            .highlight_style(theme.highlight)
            .highlight_symbol(if self.mode == PopupMode::Edit { "> " } else { "" });

        frame.render_stateful_widget(list, layout[1], &mut self.list_state);

        // Clear any previous errors if rendering was successful
        self.last_error = None;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{HighlightConfig, QueryConfig};
    use ratatui::layout::Rect;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn create_test_config() -> Config {
        Config {
            query: QueryConfig {
                category: "quant-ph".to_string(),
            },
            highlight: HighlightConfig {
                keywords: Some(vec!["quantum".to_string(), "entanglement".to_string()]),
                authors: Some(vec!["Einstein".to_string(), "Bohr".to_string()]),
            },
        }
    }

    #[test]
    fn test_popup_visibility() {
        let mut popup = ConfigPopup::new();
        
        // Initially not visible
        assert!(!popup.is_visible());
        
        // Toggle to visible
        popup.toggle();
        assert!(popup.is_visible());
        
        // Toggle back to not visible
        popup.toggle();
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_popup_mode() {
        let mut popup = ConfigPopup::new();
        
        // Initially in view mode
        assert_eq!(popup.mode(), PopupMode::View);
        
        // Toggle visibility resets mode to view
        popup.toggle();
        assert_eq!(popup.mode(), PopupMode::View);
        
        // Toggle visibility again
        popup.toggle();
        assert_eq!(popup.mode(), PopupMode::View);
    }

    #[test]
    fn test_popup_selection() {
        let mut popup = ConfigPopup::new();
        
        // Initially no selection
        assert_eq!(popup.selected_index(), None);
        
        // Toggle visibility resets selection
        popup.toggle();
        assert_eq!(popup.selected_index(), None);
        
        // Toggle visibility again
        popup.toggle();
        assert_eq!(popup.selected_index(), None);
    }

    #[test]
    fn test_popup_rendering() {
        let mut popup = ConfigPopup::new();
        let theme = Theme::default();
        let config = create_test_config();
        
        // Create a test buffer
        let area = Rect::new(0, 0, 100, 50);
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();
        
        // Test rendering when not visible
        assert!(!popup.is_visible());
        
        // Test rendering when visible
        popup.toggle();
        let result = terminal.draw(|frame| {
            if let Err(e) = popup.render(frame, area, &theme, &config) {
                panic!("Failed to render popup: {}", e);
            }
        });
        assert!(result.is_ok());
        assert!(popup.is_visible());
        assert_eq!(popup.mode(), PopupMode::View);
        assert_eq!(popup.selected_index(), None);
    }

    #[test]
    fn test_popup_with_empty_config() {
        let mut popup = ConfigPopup::new();
        let theme = Theme::default();
        let config = Config::default(); // Empty config
        
        // Create a test buffer
        let area = Rect::new(0, 0, 100, 50);
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();
        
        // Test rendering with empty config
        popup.toggle();
        let result = terminal.draw(|frame| {
            if let Err(e) = popup.render(frame, area, &theme, &config) {
                panic!("Failed to render popup: {}", e);
            }
        });
        assert!(result.is_ok());
        assert!(popup.is_visible());
        assert_eq!(popup.mode(), PopupMode::View);
        assert_eq!(popup.selected_index(), None);
    }

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let rect = centered_rect(60, 40, area);
        
        // Verify the rect is centered
        assert_eq!(rect.x, 20); // (100 - 60) / 2
        assert_eq!(rect.y, 15); // (50 - 40) / 2
        assert_eq!(rect.width, 60);
        assert_eq!(rect.height, 20);
    }

    #[test]
    fn test_popup_error_handling() {
        let mut popup = ConfigPopup::new();
        let theme = Theme::default();
        let config = create_test_config();
        
        // Test with too small area
        let small_area = Rect::new(0, 0, 10, 5);
        let backend = TestBackend::new(10, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        
        popup.toggle();
        let result = terminal.draw(|frame| {
            let render_result = popup.render(frame, small_area, &theme, &config);
            assert!(render_result.is_err());
        });
        assert!(result.is_ok());
        assert!(popup.last_error().is_some());
        assert!(matches!(popup.last_error().unwrap(), ConfigPopupError::LayoutError(_)));
    }

    #[test]
    fn test_popup_error_clearing() {
        let mut popup = ConfigPopup::new();
        let theme = Theme::default();
        let config = create_test_config();
        
        // Create a test buffer
        let area = Rect::new(0, 0, 100, 50);
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();
        
        // First render with small area to generate error
        let small_area = Rect::new(0, 0, 10, 5);
        popup.toggle(); // Make sure popup is visible
        let _ = terminal.draw(|frame| {
            let render_result = popup.render(frame, small_area, &theme, &config);
            assert!(render_result.is_err());
        });
        
        assert!(popup.last_error().is_some());
        
        // Toggle visibility to clear error
        popup.toggle();
        assert!(popup.last_error().is_none());
        
        // Render with correct area
        let result = terminal.draw(|frame| {
            if let Err(e) = popup.render(frame, area, &theme, &config) {
                panic!("Failed to render popup: {}", e);
            }
        });
        
        assert!(result.is_ok());
        assert!(popup.last_error().is_none());
    }
} 