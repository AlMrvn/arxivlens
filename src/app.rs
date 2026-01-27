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

pub mod actions;

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
        if self.article_feed.len > 0 {
            self.article_feed.state.select(Some(0));
        }
    }

    pub fn select_last(&mut self) {
        if self.article_feed.len > 0 {
            // Safe from underflow because we checked len > 0
            self.article_feed
                .state
                .select(Some(self.article_feed.len - 1));
        }
    }

    /// Scroll down by a specified number of steps
    pub fn scroll_down(&mut self, step: usize) {
        if self.article_feed.len == 0 {
            return;
        }
        let current = self.article_feed.state.selected().unwrap_or(0);
        let new_index = (current + step).min(self.article_feed.len - 1);
        self.article_feed.state.select(Some(new_index));
    }

    /// Scroll up by a specified number of steps
    pub fn scroll_up(&mut self, step: usize) {
        if self.article_feed.len == 0 {
            return;
        }

        let current = self.article_feed.state.selected().unwrap_or(0);
        let new_index = current.saturating_sub(step);

        self.article_feed.state.select(Some(new_index));
    }

    /// Get the currently selected index for testing
    pub fn selected_index(&self) -> Option<usize> {
        self.article_feed.state.selected()
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

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn create_test_app() -> App<'static> {
        use crate::app::HighlightConfig;
        use crate::arxiv::ArxivQueryResult;
        use std::sync::OnceLock;

        static RESULTS: OnceLock<ArxivQueryResult> = OnceLock::new();
        static HIGHLIGHT: OnceLock<HighlightConfig> = OnceLock::new();

        let results = RESULTS.get_or_init(|| {
            let mut entries = Vec::new();

            for i in 1..=5 {
                let mut entry = crate::arxiv::ArxivEntry::default();
                entry.title = format!("Paper {}", i);
                entry.authors = vec!["Alice".into()];
                entries.push(entry);
            }

            ArxivQueryResult {
                articles: entries,
                ..Default::default()
            }
        });

        let highlight = HIGHLIGHT.get_or_init(|| {
            HighlightConfig {
                // Option expects Some(Vec<String>)
                authors: Some(vec!["Alice".into()]),
                keywords: Some(vec!["TUI".into()]),
            }
        });

        App::new(
            results,
            highlight,
            crate::app::Theme::default(),
            crate::app::Config::default(),
        )
    }
    #[test]
    fn test_app_creation() {
        let app = create_test_app();
        assert!(app.running);
        assert_eq!(app.query_result.articles.len(), 5);
    }

    #[test]
    fn test_scroll_methods() {
        let mut app = create_test_app();

        // Test scroll down
        app.scroll_down(2);
        assert_eq!(app.selected_index(), Some(2));

        // Test scroll up
        app.scroll_up(1);
        assert_eq!(app.selected_index(), Some(1));
    }
}
