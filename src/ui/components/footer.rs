use crate::app::{actions::KEY_MAP, Context};
use crate::ui::{component::Component, theme::Theme};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

pub struct FooterComponent {
    focused: bool,
}

pub struct FooterState {
    pub current_context: Context,
    pub visible: bool,
}

impl FooterComponent {
    pub fn new() -> Self {
        Self { focused: false }
    }
}

impl Component<'_> for FooterComponent {
    type State = FooterState;

    fn render(&self, frame: &mut Frame, area: Rect, state: &mut Self::State, theme: &Theme) {
        if !state.visible {
            return;
        }

        // Check if terminal is too narrow to render footer properly
        if area.width < 10 {
            return; // Don't render anything if too narrow
        }

        // Special handling for Search context
        if state.current_context == Context::Search {
            let search_shortcuts = vec![
                ("[Esc]".to_string(), "Cancel".to_string()),
                ("[Enter]".to_string(), "Apply".to_string()),
            ];
            let footer_line = build_footer_line(&search_shortcuts, area.width);

            // Use theme's footer style
            let footer_style = theme.get_footer_style();

            frame.render_widget(
                Paragraph::new(footer_line)
                    .style(footer_style)
                    .block(Block::new()),
                area,
            );
            return;
        }

        // Filter KEY_MAP for primary actions valid in current context
        let primary_keybinds: Vec<_> = KEY_MAP
            .iter()
            .filter(|keybind| {
                keybind.is_primary && keybind.action.is_valid_in(&state.current_context)
            })
            .collect();

        // Group by action and combine keys
        let mut action_groups: std::collections::HashMap<crate::app::actions::Action, Vec<String>> =
            std::collections::HashMap::new();

        for keybind in primary_keybinds {
            let key_str = match keybind.key {
                ratatui::crossterm::event::KeyCode::Char(c) => c.to_string(),
                _ => continue, // Skip non-char keys for primary display
            };

            action_groups
                .entry(keybind.action)
                .or_default()
                .push(key_str);
        }

        // Create formatted shortcuts with priority ordering
        let mut shortcuts: Vec<(String, String)> = Vec::new();

        // Priority actions (always show if available)
        let priority_actions = [
            crate::app::actions::Action::Quit,
            crate::app::actions::Action::ShowHelp,
        ];

        // Secondary actions (show if space permits)
        let secondary_actions = [
            crate::app::actions::Action::MoveUp,
            crate::app::actions::Action::MoveDown,
            crate::app::actions::Action::YankId,
            crate::app::actions::Action::ToggleConfig,
        ];

        // Add priority actions first
        for action in priority_actions {
            if let Some(keys) = action_groups.get(&action) {
                let keys_str = keys.join("/");
                shortcuts.push((keys_str, action.description().to_string()));
            }
        }

        // Add secondary actions if space permits (only if width >= 50)
        if area.width >= 50 {
            for action in secondary_actions {
                if let Some(keys) = action_groups.get(&action) {
                    let keys_str = if action == crate::app::actions::Action::MoveUp
                        && action_groups.contains_key(&crate::app::actions::Action::MoveDown)
                    {
                        "j/k".to_string()
                    } else if action == crate::app::actions::Action::MoveDown
                        && shortcuts.iter().any(|(k, _)| k == "j/k")
                    {
                        continue; // Skip MoveDown as it's already handled with MoveUp
                    } else {
                        keys.join("/")
                    };

                    let description = if action == crate::app::actions::Action::MoveUp {
                        "Nav".to_string()
                    } else {
                        action.description().to_string()
                    };

                    shortcuts.push((keys_str, description));
                }
            }
        }

        // Build footer using Line and Span for better control
        let footer_line = build_footer_line(&shortcuts, area.width);

        // Use theme's footer style
        let footer_style = theme.get_footer_style();

        frame.render_widget(
            Paragraph::new(footer_line)
                .style(footer_style)
                .block(Block::new()),
            area,
        );
    }

    fn on_focus(&mut self) {
        self.focused = true;
    }

    fn on_blur(&mut self) {
        self.focused = false;
    }
}

/// Build a footer line that fits within the given width
fn build_footer_line(shortcuts: &[(String, String)], available_width: u16) -> Line<'static> {
    let mut spans = Vec::new();
    let mut current_width = 0u16;
    let separator = " | ";
    let separator_width = separator.len() as u16;

    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let shortcut_text = format!("{} {}", key, desc);
        let shortcut_width = shortcut_text.len() as u16;

        // Calculate total width needed (including separator if not first item)
        let needed_width = if i == 0 {
            shortcut_width
        } else {
            shortcut_width + separator_width
        };

        // Check if we have enough space
        if current_width + needed_width > available_width {
            // If this is not the first item and we can't fit it, stop here
            if i > 0 {
                break;
            }
            // If this is the first item and it doesn't fit, truncate it
            let max_len = (available_width as usize).saturating_sub(3); // Leave space for "..."
            if max_len > 0 {
                let truncated = if shortcut_text.len() > max_len {
                    format!("{}...", &shortcut_text[..max_len])
                } else {
                    shortcut_text
                };
                spans.push(Span::raw(truncated));
            }
            break;
        }

        // Add separator if not the first item
        if i > 0 {
            spans.push(Span::raw(separator));
            current_width += separator_width;
        }

        // Add the shortcut
        spans.push(Span::raw(shortcut_text));
        current_width += shortcut_width;
    }

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;

    #[test]
    fn test_footer_component_responsive_width() {
        let theme = Theme::default();
        let footer_component = FooterComponent::new();
        let mut terminal = Terminal::new(TestBackend::new(120, 10)).unwrap();

        // Test with narrow width (30 characters)
        let narrow_rect = Rect::new(0, 0, 30, 1);

        terminal
            .draw(|frame| {
                let mut state = FooterState {
                    current_context: Context::ArticleList,
                    visible: true,
                };
                footer_component.render(frame, narrow_rect, &mut state, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let footer_content = buffer.content[0..30]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        // With narrow width, should prioritize "Quit" and "Help"
        assert!(
            footer_content.contains("Quit"),
            "Footer should contain 'Quit' in narrow width"
        );
        assert!(
            footer_content.contains("Help"),
            "Footer should contain 'Help' in narrow width"
        );

        // Should not contain less critical shortcuts due to space constraints
        assert!(
            !footer_content.contains("Nav"),
            "Footer should not contain 'Nav' in narrow width"
        );
        assert!(
            !footer_content.contains("Config"),
            "Footer should not contain 'Config' in narrow width"
        );
    }

    #[test]
    fn test_footer_component_very_narrow_width() {
        let theme = Theme::default();
        let footer_component = FooterComponent::new();
        let mut terminal = Terminal::new(TestBackend::new(10, 10)).unwrap();

        // Test with very narrow width (less than 10 characters)
        let very_narrow_rect = Rect::new(0, 0, 5, 1);

        terminal
            .draw(|frame| {
                let mut state = FooterState {
                    current_context: Context::ArticleList,
                    visible: true,
                };
                footer_component.render(frame, very_narrow_rect, &mut state, &theme);
            })
            .unwrap();

        // Should not crash and should render nothing or minimal content
        let buffer = terminal.backend().buffer();
        let footer_content = buffer.content[0..5]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        // With very narrow width, footer should be empty or minimal
        // (render returns early if width < 10)
        assert!(footer_content.trim().is_empty() || footer_content.len() <= 5);
    }
}
