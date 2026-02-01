use crate::app::actions::KEY_MAP;
use crate::ui::component::{Component, TestableComponent};
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Block, Clear, Row, Table},
};

#[derive(Debug, Clone, Default)]
pub struct HelpPopupComponent {
    focused: bool,
}

pub struct HelpPopupState {
    pub visible: bool,
}

impl HelpPopupComponent {
    pub fn new() -> Self {
        Self { focused: false }
    }
}

impl Component<'_> for HelpPopupComponent {
    type State = HelpPopupState;

    fn render(&self, frame: &mut Frame, area: Rect, state: &mut Self::State, theme: &Theme) {
        if !state.visible {
            return;
        }

        // Use the theme's built-in centered_popup_area (instead of the legacy helper)
        let popup_area = theme.centered_popup_area(80, 70, area);

        // 1. Important: Clear the background
        frame.render_widget(Clear, popup_area);

        // 2. Format rows
        let rows: Vec<Row> = KEY_MAP
            .iter()
            .map(|kb| {
                let key_str = format_key(kb.key, kb.modifiers);
                let description = kb.action.description();
                let primary_indicator = if kb.is_primary { " *" } else { "" };

                Row::new(vec![
                    format!(" {} ", key_str),
                    format!(" {}{} ", description, primary_indicator),
                ])
                .style(theme.popup.background)
            })
            .collect();

        // 3. Render as a Table for better alignment than the old List
        let table = Table::new(
            rows,
            [Constraint::Percentage(30), Constraint::Percentage(70)],
        )
        .block(
            Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_set(theme.border.set)
                .border_style(theme.get_border_style(self.focused, true))
                .title(" Help - All Key Bindings (* = footer) ")
                .title_style(theme.popup.title),
        )
        .header(
            Row::new(vec![" Key", " Action"])
                .style(theme.help.key)
                .bottom_margin(1),
        );

        frame.render_widget(table, popup_area);
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

/// Ported from legacy for clean key rendering
fn format_key(
    key: ratatui::crossterm::event::KeyCode,
    modifiers: ratatui::crossterm::event::KeyModifiers,
) -> String {
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};
    let key_str = match key {
        KeyCode::Char(c) => {
            if modifiers.contains(KeyModifiers::SHIFT) && c.is_ascii_lowercase() {
                c.to_ascii_uppercase().to_string()
            } else {
                c.to_string()
            }
        }
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::Enter => "⏎".to_string(),
        KeyCode::Backspace => "⌫".to_string(),
        _ => format!("{:?}", key),
    };

    if modifiers.contains(KeyModifiers::CONTROL) {
        format!("Ctrl+{}", key_str)
    } else {
        key_str
    }
}

impl TestableComponent<'_> for HelpPopupComponent {
    fn create_test_instance() -> Self {
        Self::new()
    }

    fn get_test_state() -> Self::State {
        HelpPopupState { visible: true }
    }

    fn test_name() -> &'static str {
        "help_popup"
    }
}
