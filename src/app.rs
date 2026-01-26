use crate::arxiv::ArxivQueryResult;
use crate::config::{Config, HighlightConfig};
use crate::ui::{option_vec_to_option_slice, ArticleDetails, ArticleFeed, ConfigPopup, Theme};
use arboard::Clipboard;
use std::error::Error;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Paragraph},
    Frame,
};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn Error>>;

/// Application.
#[derive(Debug)]
pub struct App<'a> {
    /// Is the application running?
    pub running: bool,
    /// Arxiv entry list:
    pub query_result: &'a ArxivQueryResult,
    /// Configuration for the hilighting
    pub highlight_config: &'a HighlightConfig,
    /// The title of articles feeds
    pub article_feed: ArticleFeed<'a>,
    /// Theme
    pub theme: Theme,
    /// Configuration popup
    pub config_popup: ConfigPopup,
    /// Configuration
    pub config: Config,
}

impl<'a> App<'a> {
    pub fn new(
        query_result: &'a ArxivQueryResult,
        highlight_config: &'a HighlightConfig,
        theme: Theme,
        config: Config,
    ) -> Self {
        // Constructing the highlighed feed of titles.
        let patterns = option_vec_to_option_slice(&highlight_config.authors);
        let article_feed = ArticleFeed::new(query_result, patterns.as_deref(), &theme);

        Self {
            running: true,
            query_result,
            highlight_config,
            article_feed,
            theme,
            config_popup: ConfigPopup::new(),
            config,
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
        self.article_feed.state.select(None)
    }

    /// Select next item:
    pub fn select_next(&mut self) {
        self.article_feed.state.select_next();
    }
    pub fn select_previous(&mut self) {
        self.article_feed.state.select_previous();
    }

    pub fn select_first(&mut self) {
        self.article_feed.state.select_first();
    }

    pub fn select_last(&mut self) {
        self.article_feed.state.select_last();
    }

    pub fn yank_id(&mut self) {
        // The abstract of the manuscript
        let id = if let Some(i) = self.article_feed.state.selected() {
            self.query_result.articles[i].id.clone()
        } else {
            "Nothing selected".to_string()
        };

        // Set the clipboard
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.set_text(id).unwrap();
    }

    /// Toggle the configuration popup
    pub fn toggle_config(&mut self) {
        self.config_popup.toggle();
    }

    /// Render the app:
    pub fn render(&mut self, frame: &mut Frame) {
        // First we create a Layout
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Min(1)])
            .split(frame.size());

        // adding the shortcut
        frame.render_widget(
            Paragraph::new("   quit: q  |  up: k  | down: j | yank url: y | config: c")
                .style(self.theme.shortcut)
                .left_aligned()
                .block(Block::new()),
            layout[1],
        );

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(2)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[0]);

        // Render the slectable feed
        self.article_feed.render(frame, layout[0]);

        // Render the detail of the article selected:
        let current_entry = if let Some(i) = self.article_feed.state.selected() {
            &self.query_result.articles[i]
        } else {
            // Should implement a default print here ?
            &self.query_result.articles[0]
        };

        let article_view = ArticleDetails::new(current_entry, self.highlight_config, &self.theme);
        article_view.render(frame, layout[1], &self.theme);

        // Render the config popup if visible
        if self.config_popup.is_visible() {
            if let Err(e) = self
                .config_popup
                .render(frame, frame.size(), &self.theme, &self.config)
            {
                // Log the error but don't crash
                eprintln!("Error rendering config popup: {e}");
            }
        }
    }
}
