use crate::arxiv::{ArxivEntry, ArxivQueryResult};
use crate::config::{Config, HighlightConfig};
use crate::search::engine::SearchEngine;
use crate::ui::{
    option_vec_to_option_slice, render_footer, render_help_popup, search::render_search_bar,
    ArticleDetails, ArticleFeed, ConfigPopup, Theme,
};
use arboard::Clipboard;
use search::SearchState;
use std::error::Error;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub mod actions;
pub mod search;

/// Application context enum to track current UI state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    ArticleList,
    Config,
    Help,
    Search,
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
    /// Search state
    pub search_state: SearchState,
    /// Current selection state for the article list
    pub article_list_state: ratatui::widgets::ListState,
    /// Search Engine for fuzzy-matching
    pub search_engine: SearchEngine,
}

impl<'a> App<'a> {
    pub fn new(
        query_result: &'a ArxivQueryResult,
        highlight_config: &'a HighlightConfig,
        theme: Theme,
        config: Config,
    ) -> Self {
        // Initialize search state with articles
        let mut search_state = SearchState::new();
        search_state.set_articles(&query_result.articles);

        Self {
            running: true,
            query_result,
            highlight_config,
            theme,
            config_popup: ConfigPopup::new(),
            config,
            current_context: Context::ArticleList,
            show_help: false,
            help_state: ratatui::widgets::ListState::default(),
            search_state,
            article_list_state: ratatui::widgets::ListState::default(),
            search_engine: crate::search::engine::SearchEngine::new(),
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
        self.article_list_state.select(None)
    }

    /// Select next item:
    pub fn select_next(&mut self) {
        self.article_list_state.select_next();
    }
    pub fn select_previous(&mut self) {
        self.article_list_state.select_previous();
    }

    pub fn select_first(&mut self) {
        let visible_count = self.get_visible_count();
        if visible_count > 0 {
            self.article_list_state.select(Some(0));
        }
    }

    pub fn select_last(&mut self) {
        let visible_count = self.get_visible_count();
        if visible_count > 0 {
            // Safe from underflow because we checked len > 0
            self.article_list_state.select(Some(visible_count - 1));
        }
    }

    /// Scroll down by a specified number of steps
    pub fn scroll_down(&mut self, step: usize) {
        let visible_count = self.get_visible_count();
        if visible_count == 0 {
            return;
        }
        let current = self.article_list_state.selected().unwrap_or(0);
        let new_index = (current + step).min(visible_count - 1);
        self.article_list_state.select(Some(new_index));
    }

    /// Scroll up by a specified number of steps
    pub fn scroll_up(&mut self, step: usize) {
        let visible_count = self.get_visible_count();
        if visible_count == 0 {
            return;
        }

        let current = self.article_list_state.selected().unwrap_or(0);
        let new_index = current.saturating_sub(step);

        self.article_list_state.select(Some(new_index));
    }

    /// Calculate half-page scroll step based on terminal height
    pub fn calculate_half_page_step(&self, terminal_height: u16) -> usize {
        // Ensure we have at least 1 step, even for very small terminals
        (terminal_height as usize / 2).max(1)
    }

    /// Method to centralize context transitions and handle all side effects
    pub fn set_context(&mut self, new_context: Context) {
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
            Context::Search => {
                self.current_context = Context::Search;
                // Initialize search state with articles if needed
                self.search_state.set_articles(&self.query_result.articles);
                if !self.search_state.is_active() {
                    self.search_state.clear();
                }
                // Close other popups
                self.show_help = false;
                if self.config_popup.is_visible() {
                    self.config_popup.toggle();
                }
            }
            Context::ArticleList => {
                // Try to maintain current selection when exiting search
                let current_selection = if self.current_context == Context::Search {
                    // Get the actual article index from the current filtered selection
                    self.article_list_state
                        .selected()
                        .and_then(|visible_idx| self.get_actual_article_index(visible_idx))
                } else {
                    self.article_list_state.selected()
                };

                self.current_context = Context::ArticleList;
                self.show_help = false;
                // Close config popup if open
                if self.config_popup.is_visible() {
                    self.config_popup.toggle();
                }
                // Clear search state when leaving search
                self.search_state.clear();

                // Restore selection if it was valid
                if let Some(selection) = current_selection {
                    if selection < self.query_result.articles.len() {
                        self.article_list_state.select(Some(selection));
                    }
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
            actions::Action::Search => {
                self.set_context(Context::Search);
            }
            actions::Action::ClosePopup => {
                match self.current_context {
                    Context::Config | Context::Help => {
                        self.set_context(Context::ArticleList);
                    }
                    Context::Search => {
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
        self.article_list_state.selected()
    }

    /// Check if the app is currently in search mode
    pub fn is_searching(&self) -> bool {
        self.current_context == Context::Search
    }

    /// Get the currently visible articles (filtered or all)
    pub fn get_visible_articles(&self) -> Vec<&ArxivEntry> {
        if self.search_state.is_active() {
            self.search_state
                .filtered_indices
                .iter()
                .filter_map(|&index| self.query_result.articles.get(index))
                .collect()
        } else {
            self.query_result.articles.iter().collect()
        }
    }

    /// Get the current article count (filtered or total)
    pub fn get_visible_count(&mut self) -> usize {
        if self.search_state.is_active() {
            self.update_search_filter();
            self.search_state.filtered_count()
        } else {
            self.query_result.articles.len()
        }
    }
    /// Internal helper to sync the search engine with the UI state
    pub fn update_search_filter(&mut self) {
        let haystacks: Vec<String> = self
            .query_result
            .articles
            .iter()
            .map(|a| a.title.clone())
            .collect();

        let indices = self
            .search_engine
            .filter(&self.search_state.query, &haystacks);

        // Update the search_state with the new indices
        self.search_state.filtered_indices = indices;
    }

    /// Get the actual article index from the visible index
    pub fn get_actual_article_index(&self, visible_index: usize) -> Option<usize> {
        if self.search_state.is_active() {
            self.search_state
                .filtered_indices
                .get(visible_index)
                .copied()
        } else {
            Some(visible_index)
        }
    }

    /// Ensure selection is within bounds of visible articles
    pub fn clamp_selection(&mut self) {
        let visible_count = self.get_visible_count();
        if visible_count == 0 {
            self.article_list_state.select(None);
        } else {
            let current = self.article_list_state.selected().unwrap_or(0);
            if current >= visible_count {
                self.article_list_state.select(Some(visible_count - 1));
            }
        }
    }

    /// Ensure search state is synchronized with current articles
    pub fn sync_search_state(&mut self) {
        self.search_state.set_articles(&self.query_result.articles);
    }

    /// Sync selection to first match after search changes
    fn sync_selection_to_filter(&mut self) {
        let visible_count = self.get_visible_count();
        if visible_count > 0 {
            self.article_list_state.select(Some(0));
        } else {
            self.article_list_state.select(None);
        }
    }

    /// Reset selection to first match when search changes
    pub fn reset_selection_to_first_match(&mut self) {
        self.sync_selection_to_filter();
    }

    /// Handle search character input and sync selection
    pub fn handle_search_char(&mut self, c: char) {
        self.search_state.push_char(c);
        self.sync_selection_to_filter();
    }

    /// Handle search backspace and sync selection
    pub fn handle_search_backspace(&mut self) {
        self.search_state.pop_char();
        self.sync_selection_to_filter();
    }

    pub fn yank_id(&mut self) {
        // The abstract of the manuscript
        let id = if let Some(i) = self.article_list_state.selected() {
            if let Some(actual_index) = self.get_actual_article_index(i) {
                self.query_result.articles[actual_index].id.clone()
            } else {
                "Nothing selected".to_string()
            }
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
            Context::ArticleList | Context::Help | Context::Search => {
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
            Context::ArticleList | Context::Config | Context::Search => {
                self.set_context(Context::Help);
            }
            Context::Help => {
                self.set_context(Context::ArticleList);
            }
        }
    }

    /// Render the app:
    pub fn render(&mut self, frame: &mut Frame) {
        // Split screen into search bar (if searching), main area, and footer
        let main_constraints = if self.is_searching() {
            vec![
                Constraint::Length(3), // Search bar
                Constraint::Min(0),    // Main content
                Constraint::Length(1), // Footer
            ]
        } else {
            vec![
                Constraint::Min(0),    // Main content
                Constraint::Length(1), // Footer
            ]
        };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(main_constraints)
            .split(frame.size());

        let (main_area, footer_area) = if self.is_searching() {
            // Render search bar
            render_search_bar(frame, layout[0], self, &self.theme);
            (layout[1], layout[2])
        } else {
            (layout[0], layout[1])
        };

        // Split main area horizontally for article list and details
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(2)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_area);

        // Always create a fresh ArticleFeed with current search state
        let patterns = option_vec_to_option_slice(&self.highlight_config.authors);
        let mut current_feed = ArticleFeed::new_with_search(
            self.query_result,
            patterns.as_deref(),
            &self.theme,
            Some(&self.search_state),
        );
        // Sync the selection state
        current_feed.state = self.article_list_state.clone();
        current_feed.render(frame, main_layout[0]);
        // Update the app's list state with any changes from the feed
        self.article_list_state = current_feed.state;

        // Render the detail of the article selected
        let visible_count = self.get_visible_count();
        if visible_count == 0 && self.search_state.is_active() {
            // Show "No results found" message when searching with no results
            let no_results = ArticleDetails::no_results(&self.theme);
            no_results.render(frame, main_layout[1], &self.theme);
        } else {
            let current_entry = if let Some(i) = self.article_list_state.selected() {
                if let Some(actual_index) = self.get_actual_article_index(i) {
                    &self.query_result.articles[actual_index]
                } else if self.search_state.is_active()
                    && !self.search_state.filtered_indices.is_empty()
                {
                    // Fallback to first filtered article if selection is invalid during search
                    &self.query_result.articles[self.search_state.filtered_indices[0]]
                } else {
                    &self.query_result.articles[0]
                }
            } else if self.search_state.is_active() && visible_count > 0 {
                // Get first filtered article when no selection but have results
                if let Some(&first_filtered_index) = self.search_state.filtered_indices.first() {
                    &self.query_result.articles[first_filtered_index]
                } else {
                    &self.query_result.articles[0]
                }
            } else {
                &self.query_result.articles[0]
            };

            let article_view =
                ArticleDetails::new(current_entry, self.highlight_config, &self.theme);
            article_view.render(frame, main_layout[1], &self.theme);
        }

        // Render popups
        if self.config_popup.is_visible() {
            if let Err(e) = self
                .config_popup
                .render(frame, frame.size(), &self.theme, &self.config)
            {
                eprintln!("Error rendering config popup: {e}");
            }
        }

        if self.show_help {
            render_help_popup(frame, frame.size(), self);
        }

        // Render the context-aware footer
        render_footer(frame, footer_area, self);
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
                Context::Search => {
                    self.current_context = Context::Search;
                    // Initialize search state with articles if needed
                    self.search_state.set_articles(&self.query_result.articles);
                    if !self.search_state.is_active() {
                        self.search_state.clear();
                    }
                    // Close other popups
                    self.show_help = false;
                    if self.config_popup.is_visible() {
                        self.config_popup.toggle();
                    }
                }
                Context::ArticleList => {
                    self.current_context = Context::ArticleList;
                    self.show_help = false;
                    // Close config popup if open
                    if self.config_popup.is_visible() {
                        self.config_popup.toggle();
                    }
                    // Clear search state when leaving search
                    self.search_state.clear();
                }
            }
        }
    }

    pub fn create_test_app() -> App<'static> {
        use crate::arxiv::{ArxivEntry, ArxivQueryResult};
        use crate::config::HighlightConfig;
        use std::sync::OnceLock;

        static RESULTS: OnceLock<ArxivQueryResult> = OnceLock::new();
        static HIGHLIGHT: OnceLock<HighlightConfig> = OnceLock::new();

        let results = RESULTS.get_or_init(|| {
            let mut entries = Vec::new();

            for i in 1..=5 {
                let entry = ArxivEntry::new(
                    format!("Paper {}", i),
                    vec!["Alice".to_string()],
                    format!("Summary for paper {}", i),
                    format!("id-{}", i),
                    "2023-01-01".to_string(),
                    "2023-01-01".to_string(),
                );
                entries.push(entry);
            }

            ArxivQueryResult {
                updated: "2023-01-01".to_string(),
                articles: entries,
            }
        });

        let highlight = HIGHLIGHT.get_or_init(|| HighlightConfig {
            authors: Some(vec!["Alice".to_string()]),
            keywords: Some(vec!["TUI".to_string()]),
        });

        App::new(results, highlight, Theme::default(), Config::default())
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

        // First select an item to test scrolling
        app.select_first();
        assert_eq!(app.selected_index(), Some(0));

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
    #[test]
    fn test_app_search_updates_visibility() {
        let mut app = create_test_app(); // Should have 5 papers
        let total = app.get_visible_count();

        // Simulate what the user does
        app.set_context(Context::Search);
        app.search_state.push_char('z');
        app.search_state.push_char('z');
        app.search_state.push_char('z'); // "zzz" should match nothing

        // If this unit test also fails, we've caught the bug locally.
        let visible = app.get_visible_count();

        assert!(visible < total, "Visible count should decrease");
        assert_eq!(visible, 0, "Query 'zzz' should return 0 results");
    }

    #[test]
    fn test_app_search_partial_match() {
        let mut app = create_test_app();

        // 1. Search for "Paper 1"
        app.search_state.push_char('P');
        app.search_state.push_char('a');
        app.search_state.push_char('p');
        app.search_state.push_char('e');
        app.search_state.push_char('r');
        app.search_state.push_char(' ');
        app.search_state.push_char('1');

        // 2. Verify we only see the one matching paper
        assert_eq!(app.get_visible_count(), 1);
    }

    #[test]
    fn diagnostic_replicate_integration_failure() {
        let mut app = create_test_app();
        let total = app.query_result.articles.len(); // Should be 5

        // 1. Enter Search
        app.set_context(Context::Search);

        // 2. Simulate typing 'Paper 1'
        for c in "Paper 1".chars() {
            app.search_state.push_char(c);
        }

        // 3. Check App's internal view
        let visible_count = app.get_visible_count();
        println!("DEBUG: Visible count is {}", visible_count);

        // If your integration test fails at line 97, it's likely doing this:
        // assert!(visible_count < total);

        assert!(visible_count > 0, "Should match at least one paper");
        assert!(
            visible_count < total,
            "Should have filtered out other papers"
        );
    }
}
