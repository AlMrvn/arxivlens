use crate::arxiv::ArxivQueryResult;
use crate::config::{Config, HighlightConfig};
use crate::ui::{
    option_vec_to_option_slice, render_footer, render_help_popup, ArticleDetails, ArticleFeed,
    ConfigPopup, Theme,
};
use arboard::Clipboard;
use std::error::Error;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub mod actions;

/// Application context enum to track current UI state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    ArticleList,
    Config,
    Help,
}

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
    /// Current application context
    pub current_context: Context,
    /// Whether help is currently shown
    pub show_help: bool,
    /// Help popup list state
    pub help_state: ratatui::widgets::ListState,
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
            current_context: Context::ArticleList,
            show_help: false,
            help_state: ratatui::widgets::ListState::default(),
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

    /// Calculate half-page scroll step based on terminal height
    pub fn calculate_half_page_step(&self, terminal_height: u16) -> usize {
        // Ensure we have at least 1 step, even for very small terminals
        (terminal_height as usize / 2).max(1)
    }

    /// Private method to centralize context transitions and handle all side effects
    fn set_context(&mut self, new_context: Context) {
        match new_context {
            Context::Help => {
                self.current_context = Context::Help;
                self.show_help = true;
                self.help_state.select(Some(0));
                // Close config popup if open
                if self.config_popup.is_visible() {
                    self.config_popup.toggle();
                }
            }
            Context::Config => {
                self.current_context = Context::Config;
                // Ensure config popup is visible
                if !self.config_popup.is_visible() {
                    self.config_popup.toggle();
                }
                // Close help if open
                self.show_help = false;
            }
            Context::ArticleList => {
                self.current_context = Context::ArticleList;
                self.show_help = false;
                // Close config popup if open
                if self.config_popup.is_visible() {
                    self.config_popup.toggle();
                }
            }
        }
    }

    /// Perform the given action with the provided terminal height
    pub fn perform_action(&mut self, action: actions::Action, terminal_height: u16) {
        match action {
            actions::Action::Quit => self.quit(),
            actions::Action::MoveUp => self.select_previous(),
            actions::Action::MoveDown => self.select_next(),
            actions::Action::PageUp => {
                let step = self.calculate_half_page_step(terminal_height);
                self.scroll_up(step);
            }
            actions::Action::PageDown => {
                let step = self.calculate_half_page_step(terminal_height);
                self.scroll_down(step);
            }
            actions::Action::GoToTop => self.select_first(),
            actions::Action::GoToBottom => self.select_last(),
            actions::Action::ToggleConfig => self.toggle_config(),
            actions::Action::ShowHelp => self.toggle_help(),
            actions::Action::YankId => self.yank_id(),
            actions::Action::ClosePopup => {
                match self.current_context {
                    Context::Config | Context::Help => {
                        self.set_context(Context::ArticleList);
                    }
                    Context::ArticleList => {
                        self.quit(); // Quit if no popup is open
                    }
                }
            }
        }
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
        match self.current_context {
            Context::ArticleList | Context::Help => {
                self.set_context(Context::Config);
            }
            Context::Config => {
                self.set_context(Context::ArticleList);
            }
        }
    }

    /// Toggle the help display
    pub fn toggle_help(&mut self) {
        match self.current_context {
            Context::ArticleList | Context::Config => {
                self.set_context(Context::Help);
            }
            Context::Help => {
                self.set_context(Context::ArticleList);
            }
        }
    }

    /// Render the app:
    pub fn render(&mut self, frame: &mut Frame) {
        // Split screen into main area and footer
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(frame.size());

        // Split main area horizontally for article list and details
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(2)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[0]);

        // Render the selectable feed
        self.article_feed.render(frame, main_layout[0]);

        // Render the detail of the article selected:
        let current_entry = if let Some(i) = self.article_feed.state.selected() {
            &self.query_result.articles[i]
        } else {
            // Should implement a default print here ?
            &self.query_result.articles[0]
        };

        let article_view = ArticleDetails::new(current_entry, self.highlight_config, &self.theme);
        article_view.render(frame, main_layout[1], &self.theme);

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

        // Render the help popup if visible
        if self.show_help {
            render_help_popup(frame, frame.size(), self);
        }

        // Render the context-aware footer
        render_footer(frame, layout[1], self);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    impl<'a> App<'a> {
        /// Test helper to set context and synchronize related state
        pub fn set_test_context(&mut self, context: Context) {
            match context {
                Context::Help => {
                    self.show_help = true;
                    self.current_context = Context::Help;
                    self.help_state.select(Some(0));
                    // Close config popup if open
                    if self.config_popup.is_visible() {
                        self.config_popup.toggle();
                    }
                }
                Context::Config => {
                    self.current_context = Context::Config;
                    // Ensure config popup is visible
                    if !self.config_popup.is_visible() {
                        self.config_popup.toggle();
                    }
                    // Close help if open
                    self.show_help = false;
                }
                Context::ArticleList => {
                    self.current_context = Context::ArticleList;
                    self.show_help = false;
                    // Close config popup if open
                    if self.config_popup.is_visible() {
                        self.config_popup.toggle();
                    }
                }
            }
        }
    }

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

    #[test]
    fn test_context_switching() {
        let mut app = create_test_app();

        // Initially should be in ArticleList context
        assert_eq!(app.current_context, Context::ArticleList);

        // Toggle to Config context
        app.toggle_config();
        assert_eq!(app.current_context, Context::Config);

        // Toggle back to ArticleList context
        app.toggle_config();
        assert_eq!(app.current_context, Context::ArticleList);

        // Test switching from Help context to Config
        app.current_context = Context::Help;
        app.toggle_config();
        assert_eq!(app.current_context, Context::Config);
    }

    #[test]
    fn test_help_state_reset() {
        let mut app = create_test_app();

        // Initially help should not be shown
        assert!(!app.show_help);
        assert_eq!(app.current_context, Context::ArticleList);

        // Toggle help on
        app.toggle_help();
        assert!(app.show_help);
        assert_eq!(app.current_context, Context::Help);
        assert_eq!(
            app.help_state.selected(),
            Some(0),
            "Help state should be reset to first item"
        );

        // Toggle help off
        app.toggle_help();
        assert!(!app.show_help);
        assert_eq!(app.current_context, Context::ArticleList);
    }

    #[test]
    fn test_dynamic_scrolling_math() {
        let app = create_test_app();

        // Test edge cases
        assert_eq!(
            app.calculate_half_page_step(0),
            1,
            "Should handle zero height gracefully"
        );
        assert_eq!(
            app.calculate_half_page_step(1),
            1,
            "Should have minimum step of 1"
        );
        assert_eq!(
            app.calculate_half_page_step(2),
            1,
            "Should have step of 1 for height 2"
        );
        assert_eq!(
            app.calculate_half_page_step(3),
            1,
            "Should have step of 1 for height 3"
        );
        assert_eq!(
            app.calculate_half_page_step(4),
            2,
            "Should have step of 2 for height 4"
        );

        // Test normal cases
        assert_eq!(app.calculate_half_page_step(20), 10);
        assert_eq!(app.calculate_half_page_step(50), 25);
        assert_eq!(app.calculate_half_page_step(100), 50);
    }

    #[test]
    fn test_set_test_context_helper() {
        let mut app = create_test_app();

        // Test setting Help context
        app.set_test_context(Context::Help);
        assert_eq!(app.current_context, Context::Help);
        assert!(app.show_help);
        assert_eq!(app.help_state.selected(), Some(0));
        assert!(!app.config_popup.is_visible());

        // Test setting Config context
        app.set_test_context(Context::Config);
        assert_eq!(app.current_context, Context::Config);
        assert!(!app.show_help);
        assert!(app.config_popup.is_visible());

        // Test setting ArticleList context
        app.set_test_context(Context::ArticleList);
        assert_eq!(app.current_context, Context::ArticleList);
        assert!(!app.show_help);
        assert!(!app.config_popup.is_visible());
    }
}
