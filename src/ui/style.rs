use ratatui::style::{Color, Style};

// Using the Tokyonight color palette. See https://lospec.com/palette-list/tokyo-night.
const ORANGE: Color = Color::Rgb(255, 158, 100);
const TEAL: Color = Color::Rgb(65, 166, 181);

#[derive(Debug, Clone)]
pub struct Theme {
    pub main: Style,
    pub title: Style,
    pub shortcut: Style,
    pub highlight: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            main: Style::new().fg(TEAL).bg(Color::Black),
            title: Style::new().fg(ORANGE),
            shortcut: Style::new().fg(Color::Blue).bg(Color::Black),
            highlight: Style::new().fg(TEAL).bg(Color::White),
        }
    }
}
