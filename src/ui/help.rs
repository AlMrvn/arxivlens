use crate::app::{actions::KEY_MAP, App};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

/// Render a centered help popup showing all key bindings
pub fn render_help_popup(frame: &mut Frame, area: Rect, app: &mut App) {
    // Calculate popup size (80% of screen width, 70% of height)
    let popup_area = centered_rect(80, 70, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    // Create help items from KEY_MAP
    let help_items: Vec<ListItem> = KEY_MAP
        .iter()
        .map(|keybind| {
            let key_str = format_key(keybind.key, keybind.modifiers);
            let description = keybind.action.description();
            let primary_indicator = if keybind.is_primary { " *" } else { "" };

            ListItem::new(format!(
                "{:15} {}{}",
                key_str, description, primary_indicator
            ))
        })
        .collect();

    // Create the help list
    let help_list = List::new(help_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help - All Key Bindings (* = shown in footer) ")
                .style(app.theme.main),
        )
        .style(app.theme.main);

    frame.render_stateful_widget(help_list, popup_area, &mut app.help_state);
}

/// Helper function to format key combinations
fn format_key(
    key: ratatui::crossterm::event::KeyCode,
    modifiers: ratatui::crossterm::event::KeyModifiers,
) -> String {
    let key_str = match key {
        ratatui::crossterm::event::KeyCode::Char(c) => {
            if modifiers.contains(ratatui::crossterm::event::KeyModifiers::SHIFT)
                && c.is_ascii_lowercase()
            {
                c.to_ascii_uppercase().to_string()
            } else {
                c.to_string()
            }
        }
        ratatui::crossterm::event::KeyCode::Esc => "Esc".to_string(),
        ratatui::crossterm::event::KeyCode::Up => "↑".to_string(),
        ratatui::crossterm::event::KeyCode::Down => "↓".to_string(),
        ratatui::crossterm::event::KeyCode::Left => "←".to_string(),
        ratatui::crossterm::event::KeyCode::Right => "→".to_string(),
        _ => format!("{:?}", key),
    };

    if modifiers.contains(ratatui::crossterm::event::KeyModifiers::CONTROL) {
        format!("Ctrl+{}", key_str)
    } else if modifiers.contains(ratatui::crossterm::event::KeyModifiers::SHIFT)
        && key_str.len() == 1
    {
        // For single characters, SHIFT is already handled above
        key_str
    } else if modifiers.contains(ratatui::crossterm::event::KeyModifiers::SHIFT) {
        format!("Shift+{}", key_str)
    } else {
        key_str
    }
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
