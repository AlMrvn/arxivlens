use std::error;

use crate::arxiv_entry::{get_from_arxiv, ArxivEntryList};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// counter
    pub counter: u8,
    /// Arxiv entry list:
    pub arxiv_entries: ArxivEntryList,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            counter: 0,
            arxiv_entries: get_from_arxiv().expect("Because"), // arxiv_entries: ArxivEntryList::from_iter([
                                                               //     ("A1", "A2", "A3"),
                                                               //     ("B1", "B2", "B3"),
                                                               //     ("C1", "C2", "C3"),
                                                               //     ("D1", "D2", "D3"),
                                                               // ]),
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

    pub fn increment_counter(&mut self) {
        if let Some(res) = self.counter.checked_add(1) {
            self.counter = res;
        }
    }

    pub fn decrement_counter(&mut self) {
        if let Some(res) = self.counter.checked_sub(1) {
            self.counter = res;
        }
    }
}
