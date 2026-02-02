use crate::config::Config;
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use serde::{Deserialize, Serialize};

const TEAL: Color = Color::Rgb(0, 128, 128);
const ORANGE: Color = Color::Rgb(255, 165, 0);
const DARK_GRAY: Color = Color::Rgb(64, 64, 64);
const LIGHT_GRAY: Color = Color::Rgb(128, 128, 128);

/// Centralized theme configuration for the entire application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    // Core text styles
    pub main: Style,
    pub title: Style,
    pub shortcut: Style,
    pub highlight: Style,
    pub selection: Style,

    // Border and layout styles
    pub border: BorderTheme,
    pub layout: LayoutTheme,

    // Component-specific themes
    pub list: ListTheme,
    pub popup: PopupTheme,
    pub search: SearchTheme,
    pub help: HelpTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderTheme {
    pub normal: Style,
    pub focused: Style,
    pub inactive: Style,
    #[serde(skip)]
    pub set: border::Set,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutTheme {
    pub margin: u16,
    pub padding: u16,
    pub min_popup_width: u16,
    pub min_popup_height: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTheme {
    pub item: Style,
    pub selected: Style,
    pub selected_focused: Style,
    pub selected_unfocused: Style,
    pub highlighted: Style,
    pub authors: Style,
    pub date: Style,
    pub scrollbar: Style,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopupTheme {
    pub background: Style,
    pub border: Style,
    pub title: Style,
    pub shadow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTheme {
    pub input: Style,
    pub placeholder: Style,
    pub match_highlight: Style,
    pub no_results: Style,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpTheme {
    pub key: Style,
    pub description: Style,
    pub section_title: Style,
    pub separator: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            main: Style::new().fg(TEAL).bg(Color::Black),
            title: Style::new().fg(ORANGE).add_modifier(Modifier::BOLD),
            shortcut: Style::new().fg(ORANGE).add_modifier(Modifier::BOLD),
            highlight: Style::new().fg(ORANGE).bg(Color::Black),
            selection: Style::new().fg(Color::Black).bg(Color::White),

            border: BorderTheme {
                normal: Style::new().fg(LIGHT_GRAY),
                focused: Style::new().fg(ORANGE),
                inactive: Style::new().fg(DARK_GRAY),
                set: border::ROUNDED,
            },

            layout: LayoutTheme {
                margin: 1,
                padding: 1,
                min_popup_width: 40,
                min_popup_height: 10,
            },

            list: ListTheme {
                item: Style::new().fg(TEAL).add_modifier(Modifier::BOLD),
                authors: Style::new().fg(DARK_GRAY),
                selected: Style::new().fg(Color::Black).bg(Color::White),
                selected_focused: Style::new().fg(Color::Black).bg(Color::White),
                selected_unfocused: Style::new().fg(Color::White).bg(DARK_GRAY),
                highlighted: Style::new().fg(ORANGE).add_modifier(Modifier::BOLD),
                date: Style::new().fg(DARK_GRAY),
                scrollbar: Style::new().fg(LIGHT_GRAY),
            },

            popup: PopupTheme {
                background: Style::new().bg(Color::Black),
                border: Style::new().fg(ORANGE),
                title: Style::new().fg(ORANGE).add_modifier(Modifier::BOLD),
                shadow: true,
            },

            search: SearchTheme {
                input: Style::new().fg(Color::White).bg(Color::Black),
                placeholder: Style::new().fg(DARK_GRAY),
                match_highlight: Style::new().fg(Color::Black).bg(ORANGE),
                no_results: Style::new().fg(Color::Red),
            },

            help: HelpTheme {
                key: Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                description: Style::new().fg(Color::White),
                section_title: Style::new().fg(ORANGE).add_modifier(Modifier::BOLD),
                separator: Style::new().fg(DARK_GRAY),
            },
        }
    }
}

impl Theme {
    /// Load theme from configuration file
    pub fn from_config(config: &Config) -> Self {
        match config.ui.theme_name.as_str() {
            "light" => Self {
                // Initialize 'main' directly and pull the rest from Default
                main: Style::new().fg(Color::Black).bg(Color::White),
                ..Self::default()
            },
            _ => Self::default(), // Default is your "dark" TokyoNight theme
        }
    }

    /// Get border style based on focus state
    pub fn get_border_style(&self, focused: bool, active: bool) -> Style {
        match (focused, active) {
            (true, true) => self.border.focused,
            (_, true) => self.border.normal,
            _ => self.border.inactive,
        }
    }

    /// Get footer style with reversed colors
    pub fn get_footer_style(&self) -> Style {
        Style::default()
            .bg(self.main.fg.unwrap_or(ratatui::style::Color::White))
            .fg(self.main.bg.unwrap_or(ratatui::style::Color::Black))
    }

    /// Create a centered popup area with theme-aware sizing
    pub fn centered_popup_area(
        &self,
        percent_x: u16,
        percent_y: u16,
        area: ratatui::layout::Rect,
    ) -> ratatui::layout::Rect {
        use ratatui::layout::{Constraint, Direction, Layout};

        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
    /// Creates a styled title line with an optional shortcut number
    pub fn format_title<'a>(
        &self,
        title: &'a str,
        shortcut: Option<usize>,
        focused: bool,
        count: Option<usize>,
    ) -> Line<'a> {
        let mut spans = Vec::new();

        // 1. Add shortcut: "[1] "
        if let Some(n) = shortcut {
            spans.push(Span::styled(format!("[{}] ", n), self.shortcut));
        }

        // 2. Add the main title text: "Feed Name"
        spans.push(Span::styled(title.trim(), self.title));

        // 3. Item Count: Only show if provided (e.g., Some(15))
        if let Some(c) = count {
            spans.push(Span::styled(
                format!(" ({})", c),
                self.list.authors, // Subtle dimmed style
            ));
        }

        // 4. Focus indicator: " (focused)"
        if focused {
            spans.push(Span::styled(" (focused)", self.border.focused));
        }

        Line::from(spans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.layout.margin, 1);
        assert_eq!(theme.layout.padding, 1);
    }

    #[test]
    fn test_border_style_selection() {
        let theme = Theme::default();

        // Focused and active should use focused style
        let focused_style = theme.get_border_style(true, true);
        assert_eq!(focused_style, theme.border.focused);

        // Not focused but active should use normal style
        let normal_style = theme.get_border_style(false, true);
        assert_eq!(normal_style, theme.border.normal);

        // Inactive should use inactive style
        let inactive_style = theme.get_border_style(false, false);
        assert_eq!(inactive_style, theme.border.inactive);
    }
}
