use arboard::Clipboard;
use std::error;

use crate::arxiv_entry::{get_from_arxiv, ArxivEntryList};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Arxiv entry list:
    pub arxiv_entries: ArxivEntryList,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            arxiv_entries: get_from_arxiv().expect("Error while retrieving arxiv entries."),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// No selection
    pub fn select_none(&mut self) {
        self.arxiv_entries.state.select(None)
    }

    /// Select next item:
    pub fn select_next(&mut self) {
        self.arxiv_entries.state.select_next();
    }
    pub fn select_previous(&mut self) {
        self.arxiv_entries.state.select_previous();
    }

    pub fn select_first(&mut self) {
        self.arxiv_entries.state.select_first();
    }

    pub fn select_last(&mut self) {
        self.arxiv_entries.state.select_last();
    }

    pub fn yank_id(&mut self) {
        // The abstract of the manuscript
        let id = if let Some(i) = self.arxiv_entries.state.selected() {
            self.arxiv_entries.items[i].id.clone()
        } else {
            "Nothin selected".to_string()
        };

        // Set the clipboard
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.set_text(id).unwrap();
    }
}
