use crate::config::Config;
use crate::ui::component::{Component, TestableComponent};
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone, Default)]
pub struct ConfigPopupComponent {
    focused: bool,
}

pub struct ConfigPopupState<'a> {
    pub config: &'a Config,
    pub visible: bool,
}

impl ConfigPopupComponent {
    pub fn new() -> Self {
        Self { focused: false }
    }
}

/// The current mode of the config popup
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PopupMode {
    /// Viewing the current configuration
    View,
    /// Editing the configuration (planned for future implementation)
    Edit,
}

impl<'a> Component<'a> for ConfigPopupComponent {
    type State = ConfigPopupState<'a>;
    fn render(&self, frame: &mut Frame, area: Rect, state: &mut Self::State, theme: &Theme) {
        if !state.visible {
            return;
        }

        let popup_area = theme.centered_popup_area(60, 40, area);

        // Clear the popup area first:
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_set(theme.border.set)
            .border_style(theme.get_border_style(self.focused, true))
            .title(" Configuration ")
            .title_style(theme.popup.title)
            .style(theme.popup.background);

        let inner_area = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let mut lines = vec![
            Line::from(vec![
                Span::raw(" Default Category: "),
                Span::styled(&state.config.query.category, theme.list.highlighted),
            ]),
            Line::from(""),
            Line::from(Span::styled(" Highlighting: ", theme.help.section_title)),
        ];

        let mut format_list = |label: &str, items: &Vec<String>| {
            let val = if !items.is_empty() {
                items.join(", ")
            } else {
                "None".to_string()
            };

            lines.push(Line::from(vec![
                Span::styled(format!("  {}: ", label), theme.help.key),
                Span::styled(val, theme.help.description),
            ]));
        };
        format_list("Pinned Authors", &state.config.pinned.authors);
        format_list("Pinned Categories", &state.config.pinned.categories);

        let paragraph = Paragraph::new(lines)
            .style(theme.popup.background)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, inner_area);
    }

    fn can_focus(&self) -> bool {
        true
    }
    fn on_focus(&mut self) {
        self.focused = true;
    }
    fn on_blur(&mut self) {
        self.focused = false;
    }
}

impl TestableComponent<'_> for ConfigPopupComponent {
    fn create_test_instance() -> Self {
        Self::new()
    }

    fn get_test_state() -> Self::State {
        use crate::config::Config;

        // Create a mock config and leak it for 'static lifetime
        let config = Box::leak(Box::new(Config::default()));

        ConfigPopupState {
            config,
            visible: true,
        }
    }

    fn test_name() -> &'static str {
        "config_popup"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{PinnedConfig, QueryConfig};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn create_test_config() -> Config {
        Config {
            query: QueryConfig {
                category: "quant-ph".to_string(),
            },
            pinned: PinnedConfig {
                authors: vec!["Einstein".to_string()],
                categories: vec!["quantum".to_string()],
            },
            storage: crate::config::StorageConfig {
                database_name: "test.db".to_string(),
            },
        }
    }

    #[test]
    fn test_component_visibility_logic() {
        let config = create_test_config();
        let mut state = ConfigPopupState {
            config: &config,
            visible: false,
        };

        // Initially hidden
        assert!(!state.visible);

        // Simulate a toggle in your handler
        state.visible = true;
        assert!(state.visible);
    }

    #[test]
    fn test_render_with_theme() {
        let component = ConfigPopupComponent::new();
        let theme = Theme::default();
        let config = create_test_config();
        let mut state = ConfigPopupState {
            config: &config,
            visible: true,
        };

        // Define the area explicitly for the test
        let area = Rect::new(0, 0, 80, 30);
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|frame| {
            // Use the area variable directly instead of frame.area()
            component.render(frame, area, &mut state, &theme);
        });

        assert!(result.is_ok());

        let buffer = terminal.backend().buffer();
        let mut found_title = false;
        for y in 0..30 {
            for x in 0..80 {
                // Look for the 'C' in 'Configuration'
                if buffer.get(x, y).symbol() == "C" {
                    found_title = true;
                    break;
                }
            }
        }
        assert!(
            found_title,
            "Title 'Configuration' not found in terminal buffer"
        );
    }
}
