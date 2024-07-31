use crate::arxiv::ArxivQueryResult;
use crate::config::HighlightConfig;
use arboard::Clipboard;
use ratatui::widgets::ListState;
use std::error::Error;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn Error>>;

/// Application.
#[derive(Debug)]
pub struct App<'a> {
    /// Is the application running?
    pub running: bool,
    /// Arxiv entry list:
    pub query_result: ArxivQueryResult,
    /// State selectec
    pub state: ListState,
    /// Configuration for the hilighting
    pub highlight_config: &'a HighlightConfig,
}

impl<'a> App<'a> {
    pub fn new(query_result: ArxivQueryResult, highlight_config: &'a HighlightConfig) -> Self {
        Self {
            running: true,
            query_result,
            state: ListState::default(),
            highlight_config,
        }
    }
}

impl App<'_> {
    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// No selection
    pub fn select_none(&mut self) {
        self.state.select(None)
    }

    /// Select next item:
    pub fn select_next(&mut self) {
        self.state.select_next();
    }
    pub fn select_previous(&mut self) {
        self.state.select_previous();
    }

    pub fn select_first(&mut self) {
        self.state.select_first();
    }

    pub fn select_last(&mut self) {
        self.state.select_last();
    }

    pub fn yank_id(&mut self) {
        // The abstract of the manuscript
        let id = if let Some(i) = self.state.selected() {
            self.query_result.articles[i].id.clone()
        } else {
            "Nothing selected".to_string()
        };

        // Set the clipboard
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.set_text(id).unwrap();
    }
}
