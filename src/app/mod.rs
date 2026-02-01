use crate::arxiv::{ArxivEntry, ArxivQueryResult};
use crate::config::{Config, HighlightConfig};
use crate::search::engine::SearchEngine;
use crate::ui::components::config_popup::ConfigPopupComponent;
use crate::ui::Theme;
use arboard::Clipboard;
use search::SearchState;
use std::error::Error;

use ratatui::Frame;

pub mod actions;
pub mod search;

/// Application context enum to track current UI state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Context {
    ArticleList,
    Config,
    Help,
    Search,
}

/// Search action for centralized search handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchAction {
    PushChar(char),
    PopChar,
    Clear,
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
    pub config_popup: ConfigPopupComponent,
    /// Configuration
    pub config: Config,
    /// Current application context
    pub current_context: Context,
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
            config_popup: ConfigPopupComponent::new(),
            config,
            current_context: Context::ArticleList,
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
        // Handle special logic when transitioning from search context
        if self.current_context == Context::Search && new_context == Context::ArticleList {
            // Get the actual global index of the currently selected filtered item
            let actual_selection = self
                .article_list_state
                .selected()
                .and_then(|visible_idx| self.get_actual_article_index(visible_idx));

            // Clear search state when leaving search
            self.search_state.clear();

            // Restore selection to the actual global index if it was valid
            if let Some(global_index) = actual_selection {
                if global_index < self.query_result.articles.len() {
                    self.article_list_state.select(Some(global_index));
                }
            }
        }

        // Handle search initialization
        if new_context == Context::Search {
            self.search_state.set_articles(&self.query_result.articles);
            if !self.search_state.is_active() {
                self.search_state.clear();
            }
        }

        // Update the context - this is the single source of truth
        self.current_context = new_context;
    }

    /// Perform the given action with the provided terminal height
    pub fn perform_action(&mut self, action: actions::Action, terminal_height: u16) {
        match action {
            actions::Action::Quit => self.quit(),
            actions::Action::MoveUp => self.article_list_state.select_previous(),
            actions::Action::MoveDown => self.article_list_state.select_next(),
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
            actions::Action::ToggleFocus => self.toggle_focus(),
            actions::Action::ClosePopup => {
                if self.current_context == Context::ArticleList {
                    self.quit(); // Quit if no popup is open
                } else {
                    self.set_context(Context::ArticleList);
                }
            }
            actions::Action::SearchInput(search_act) => {
                match search_act {
                    SearchAction::PushChar(c) => self.search_state.push_char(c),
                    SearchAction::PopChar => self.search_state.pop_char(),
                    SearchAction::Clear => self.search_state.clear(),
                }
                self.update_search_filter();
                self.sync_selection_to_filter();
            }
        }
    }

    /// Get the currently selected index for testing
    pub fn selected_index(&self) -> Option<usize> {
        self.article_list_state.selected()
    }

    /// Toggle focus between components
    pub fn toggle_focus(&mut self) {
        use tracing::info;

        match self.current_context {
            Context::ArticleList => {
                self.set_context(Context::Search);
                info!("Focus toggled: currently focusing SearchBar");
            }
            Context::Search => {
                self.set_context(Context::ArticleList);
                info!("Focus toggled: currently focusing ArticleList");
            }
            _ => {} // No focus toggle for other contexts
        }
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
        self.update_search_filter();
        self.sync_selection_to_filter();
    }

    /// Handle search backspace and sync selection
    pub fn handle_search_backspace(&mut self) {
        self.search_state.pop_char();
        self.update_search_filter();
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
        if self.current_context == Context::Config {
            self.set_context(Context::ArticleList);
        } else {
            self.set_context(Context::Config);
        }
    }

    /// Toggle the help display
    pub fn toggle_help(&mut self) {
        if self.current_context == Context::Help {
            self.set_context(Context::ArticleList);
        } else {
            self.set_context(Context::Help);
        }
    }

    /// Render the app:
    pub fn render(&mut self, frame: &mut Frame) {
        crate::ui::render(self, frame);
    }

    /// Get the currently selected article for display using pre-calculated values
    pub fn get_selected_article_by_index(
        &self,
        selected_index: Option<usize>,
        visible_count: usize,
    ) -> Option<&ArxivEntry> {
        if visible_count == 0
            && matches!(self.current_context, Context::Search)
            && self.search_state.is_active()
        {
            return None;
        }

        if let Some(i) = selected_index {
            if let Some(actual_index) = self.get_actual_article_index(i) {
                Some(&self.query_result.articles[actual_index])
            } else if self.search_state.is_active()
                && !self.search_state.filtered_indices.is_empty()
            {
                // Fallback to first filtered article if selection is invalid during search
                Some(&self.query_result.articles[self.search_state.filtered_indices[0]])
            } else {
                Some(&self.query_result.articles[0])
            }
        } else if self.search_state.is_active() && visible_count > 0 {
            // Get first filtered article when no selection but have results
            if let Some(&first_filtered_index) = self.search_state.filtered_indices.first() {
                Some(&self.query_result.articles[first_filtered_index])
            } else {
                Some(&self.query_result.articles[0])
            }
        } else {
            Some(&self.query_result.articles[0])
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

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

        // Initially should be in ArticleList context
        assert_eq!(app.current_context, Context::ArticleList);

        // Toggle help on
        app.toggle_help();
        assert_eq!(app.current_context, Context::Help);

        // Toggle help off
        app.toggle_help();
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
