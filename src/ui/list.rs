use crate::arxiv::ArxivQueryResult;
use crate::ui::utils::check_author_match;
use crate::ui::Theme;
use ratatui::widgets::{List, ListState};
use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, HighlightSpacing, ListDirection, ListItem},
    Frame,
};

#[derive(Debug)]
pub struct ArticleFeed<'a> {
    items: List<'a>,
    pub state: ListState,
}

impl ArticleFeed<'_> {
    pub fn new(
        query_result: &ArxivQueryResult,
        highlight_authors: Option<&[&str]>,
        theme: &Theme,
    ) -> Self {
        let items: Vec<ListItem> = query_result
            .articles
            .iter()
            .map(|entry| {
                ListItem::from(entry.title.clone()).style(
                    if highlight_authors
                        .is_some_and(|patterns| check_author_match(&entry.authors, patterns))
                    {
                        theme.title
                    } else {
                        theme.main
                    },
                )
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items.clone())
            .block(
                Block::bordered()
                    .title_style(theme.title)
                    .title_alignment(Alignment::Left)
                    .title("arXiv Feed"),
            )
            .style(theme.main)
            .highlight_style(theme.selection)
            .highlight_symbol("> ")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom)
            .highlight_spacing(HighlightSpacing::Always);

        Self {
            items,
            state: ListState::default(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(&self.items, area, &mut self.state);
    }
}
