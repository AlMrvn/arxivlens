use crate::arxiv_query::{parse_arxiv_entries, query_arxiv};
use ratatui::widgets::ListState;
use std::error::Error;

#[derive(Debug, Default)]
pub struct ArxivEntry {
    pub title: String,
    pub author: String,
    pub summary: String,
}

impl ArxivEntry {
    fn new(title: &str, author: &str, summary: &str) -> Self {
        Self {
            title: title.to_string(),
            author: author.to_string(),
            summary: summary.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ArxivEntryList {
    pub items: Vec<ArxivEntry>,
    pub state: ListState,
}

impl FromIterator<(&'static str, &'static str, &'static str)> for ArxivEntryList {
    fn from_iter<T: IntoIterator<Item = (&'static str, &'static str, &'static str)>>(
        iter: T,
    ) -> Self {
        let items = iter
            .into_iter()
            .map(|(title, author, summary)| ArxivEntry::new(title, author, summary))
            .collect();
        let state = ListState::default();
        Self { items, state }
    }
}

pub fn get_from_arxiv() -> Result<ArxivEntryList, Box<dyn Error>> {
    let content = query_arxiv()?;
    let items = parse_arxiv_entries(&content)?;
    let state = ListState::default();
    Ok(ArxivEntryList { items, state })
}
