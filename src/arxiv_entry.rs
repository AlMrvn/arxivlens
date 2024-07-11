use crate::arxiv_query::{parse_arxiv_entries, query_arxiv};
use ratatui::widgets::ListState;
use std::error::Error;

#[derive(Debug, Default, PartialEq)]
pub struct ArxivEntry {
    pub title: String,
    pub authors: Vec<String>,
    pub summary: String,
    pub id: String,
    pub updated: String,
    pub published: String,
}

impl ArxivEntry {
    fn new(
        title: String,
        authors: Vec<String>,
        summary: String,
        id: String,
        updated: String,
        published: String,
    ) -> Self {
        Self {
            title,
            authors,
            summary,
            id,
            updated,
            published,
        }
    }
}

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

pub fn get_from_arxiv() -> Result<ArxivEntryList, Box<dyn Error>> {
    let content = query_arxiv()?;
    let items = parse_arxiv_entries(&content)?;
    let state = ListState::default();
    Ok(ArxivEntryList { items, state })
}
