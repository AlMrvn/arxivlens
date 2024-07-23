use crate::arxiv_parsing::{parse_arxiv_entries, ArxivEntry};
use crate::arxiv_query::{query_arxiv, SortBy, SortOrder};
use arboard::Clipboard;
use ratatui::widgets::ListState;
use std::error::Error;

/// Default values for the query:
const DEFAULT_START_INDEX: i32 = 0;
const DEFAULT_MAX_RESULTS: i32 = 200;
const DEFAULT_SORT_ORDER: SortOrder = SortOrder::Descending;
const DEFAULT_SORT_BY: SortBy = SortBy::SubmittedDate;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn Error>>;

/// an object with a state refering to the currently selected entry:
#[derive(Debug)]
pub struct ArxivEntryList {
    pub items: Vec<ArxivEntry>,
    pub state: ListState,
}

impl FromIterator<(String, Vec<String>, String, String, String, String)> for ArxivEntryList {
    fn from_iter<T: IntoIterator<Item = (String, Vec<String>, String, String, String, String)>>(
        iter: T,
    ) -> Self {
        let items = iter
            .into_iter()
            .map(|(title, authors, summary, id, updated, published)| {
                ArxivEntry::new(title, authors, summary, id, updated, published)
            })
            .collect();
        let state = ListState::default();
        Self { items, state }
    }
}

pub fn get_from_arxiv(
    category: Option<&str>,
    author: Option<&str>,
    start_index: Option<i32>,
    max_results: Option<i32>,
    sort_by: Option<SortBy>,
    sort_order: Option<SortOrder>,
) -> Result<ArxivEntryList, Box<dyn Error>> {
    let content = query_arxiv(
        category,
        author,
        start_index,
        max_results,
        sort_by,
        sort_order,
    )?;
    let items = parse_arxiv_entries(&content)?;
    let state = ListState::default();
    Ok(ArxivEntryList { items, state })
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Arxiv entry list:
    pub arxiv_entries: ArxivEntryList,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(category: Option<&str>) -> Self {
        Self {
            running: true,
            arxiv_entries: get_from_arxiv(
                category,
                None,
                Some(DEFAULT_START_INDEX),
                Some(DEFAULT_MAX_RESULTS),
                Some(DEFAULT_SORT_BY),
                Some(DEFAULT_SORT_ORDER),
            )
            .expect("Error while retrieving arxiv entries."),
        }
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
            "Nothing selected".to_string()
        };

        // Set the clipboard
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.set_text(id).unwrap();
    }
}
