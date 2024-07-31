use crate::arxiv::ArxivQueryResult;
use crate::config::HighlightConfig;
use arboard::Clipboard;
use ratatui::widgets::{List, ListState};
use std::error::Error;

use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, HighlightSpacing, ListDirection, ListItem},
};

// Using the Tokyonight color palette. See https://lospec.com/palette-list/tokyo-night.
const ORANGE: Color = Color::Rgb(255, 158, 100);
const TEAL: Color = Color::Rgb(65, 166, 181);
const HIGHLIGHT_STYLE: Style = Style::new()
    .fg(TEAL)
    .bg(Color::White)
    .add_modifier(Modifier::ITALIC);
const MAIN_STYLE: Style = Style::new().fg(TEAL).bg(Color::Black);

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
    ///
    pub listentry: List<'a>,
}

fn option_vec_to_option_slice<'a>(option_vec: &'a Option<Vec<String>>) -> Option<Vec<&'a str>> {
    let binding = option_vec
        .as_deref()
        .map(|v| v.iter().map(String::as_str).collect::<Vec<&str>>());
    binding
}

impl<'a> App<'a> {
    pub fn new(query_result: ArxivQueryResult, highlight_config: &'a HighlightConfig) -> Self {
        let patterns = option_vec_to_option_slice(&highlight_config.authors);
        let items: Vec<ListItem> = query_result
            .articles
            .iter()
            .enumerate()
            .map(|(_i, entry)| {
                ListItem::from(entry.title.clone()).style(
                    if entry.contains_author(patterns.as_deref()) {
                        Style::new().fg(ORANGE)
                    } else {
                        MAIN_STYLE
                    },
                )
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items.clone())
            .block(
                Block::bordered()
                    .title_style(Style::new().fg(ORANGE))
                    .title_alignment(Alignment::Left)
                    .title("arXiv Feed"),
            )
            .style(MAIN_STYLE)
            .highlight_style(HIGHLIGHT_STYLE)
            .highlight_symbol("> ")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom)
            .highlight_spacing(HighlightSpacing::Always);

        Self {
            running: true,
            query_result,
            state: ListState::default(),
            highlight_config,
            listentry: list,
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
