use crate::app::{actions::KEY_MAP, App};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

/// Render the footer with context-aware key bindings
pub fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    // Check if terminal is too narrow to render footer properly
    if area.width < 10 {
        return; // Don't render anything if too narrow
    }

    // Filter KEY_MAP for primary actions valid in current context
    let primary_keybinds: Vec<_> = KEY_MAP
        .iter()
        .filter(|keybind| keybind.is_primary && keybind.action.is_valid_in(&app.current_context))
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

    // Create a reversed style for the footer
    let footer_style = Style::default()
        .bg(app.theme.main.fg.unwrap_or(ratatui::style::Color::White))
        .fg(app.theme.main.bg.unwrap_or(ratatui::style::Color::Black));

    frame.render_widget(
        Paragraph::new(footer_line)
            .style(footer_style)
            .block(Block::new()),
        area,
    );
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
    use crate::app::tests::create_test_app;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;

    #[test]
    fn test_footer_responsive_width() {
        let app = create_test_app();
        let mut terminal = Terminal::new(TestBackend::new(120, 10)).unwrap();

        // Test with narrow width (30 characters)
        let narrow_rect = Rect::new(0, 0, 30, 1);

        terminal
            .draw(|frame| {
                render_footer(frame, narrow_rect, &app);
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

        // Test with wide width (120 characters)
        let wide_rect = Rect::new(0, 0, 120, 1);

        terminal
            .draw(|frame| {
                render_footer(frame, wide_rect, &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let footer_content = buffer.content[0..120]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        // With wide width, should contain all primary shortcuts
        assert!(
            footer_content.contains("Quit"),
            "Footer should contain 'Quit' in wide width"
        );
        assert!(
            footer_content.contains("Help"),
            "Footer should contain 'Help' in wide width"
        );
        assert!(
            footer_content.contains("Nav"),
            "Footer should contain 'Nav' in wide width"
        );
        assert!(
            footer_content.contains("Yank"),
            "Footer should contain 'Yank' in wide width"
        );
        assert!(
            footer_content.contains("Config"),
            "Footer should contain 'Config' in wide width"
        );
    }

    #[test]
    fn test_footer_very_narrow_width() {
        let app = create_test_app();
        let mut terminal = Terminal::new(TestBackend::new(10, 10)).unwrap();

        // Test with very narrow width (less than 10 characters)
        let very_narrow_rect = Rect::new(0, 0, 5, 1);

        terminal
            .draw(|frame| {
                render_footer(frame, very_narrow_rect, &app);
            })
            .unwrap();

        // Should not crash and should render nothing or minimal content
        let buffer = terminal.backend().buffer();
        let footer_content = buffer.content[0..5]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        // With very narrow width, footer should be empty or minimal
        // (render_footer returns early if width < 10)
        assert!(footer_content.trim().is_empty() || footer_content.len() <= 5);
    }
}
