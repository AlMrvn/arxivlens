use crate::arxiv::ArxivEntry;
use crate::search::engine::SearchEngine;
use std::collections::HashMap;
use std::fmt;

pub struct SearchState {
    /// Current search query
    pub query: String,
    /// Indices of articles that match the current query
    pub filtered_indices: Vec<usize>,
    /// Character positions to highlight for each matched article (UI layer)
    pub highlights: HashMap<usize, Vec<u32>>,
    /// Pre-rendered search strings to avoid formatting in the hot loop
    haystacks: Vec<String>,
    /// The decoupled search engine
    engine: SearchEngine,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            filtered_indices: Vec::new(),
            highlights: HashMap::new(),
            haystacks: Vec::new(),
            engine: SearchEngine::new(),
        }
    }
    /// Check if there's an active search query.
    pub fn is_active(&self) -> bool {
        !self.query.is_empty()
    }

    /// This pre-calculates the searchable strings so update_query is lightning fast.
    pub fn set_articles(&mut self, articles: &[ArxivEntry]) {
        self.haystacks = articles
            .iter()
            .map(|a| format!("{} {}", a.title, a.summary))
            .collect();
        // Reset search with new data
        self.run_search();
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.filtered_indices.clear();
        self.highlights.clear();
        // We keep the haystacks; they only change if set_articles is called.
        self.run_search();
    }

    pub fn update_query(&mut self, query: String) {
        self.query = query;
        self.run_search();
    }

    pub fn push_char(&mut self, c: char) {
        self.query.push(c);
        self.run_search();
    }

    pub fn pop_char(&mut self) {
        self.query.pop();
        self.run_search();
    }

    /// Internal logic to trigger the decoupled engine
    pub fn run_search(&mut self) {
        // Now SearchState just passes the pre-cached strings to the engine
        self.filtered_indices = self.engine.filter(&self.query, &self.haystacks);

        // Note: Highlighting logic usually lives in the UI render phase or
        // can be calculated here if Nucleo provides positions.
        // For now, we clear them to stay consistent.
        self.highlights.clear();
    }

    // --- Simple Getters ---

    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    pub fn get_article_index(&self, filtered_pos: usize) -> Option<usize> {
        self.filtered_indices.get(filtered_pos).copied()
    }
    pub fn get_rendered_titles(&self, articles: &[ArxivEntry]) -> Vec<String> {
        self.filtered_indices
            .iter()
            .map(|&idx| articles[idx].title.clone())
            .collect()
    }

    pub fn verify_indices_integrity(&self, articles: &[ArxivEntry]) -> Result<(), String> {
        for &idx in &self.filtered_indices {
            if idx >= articles.len() {
                return Err(format!("Index {} out of bounds", idx));
            }
        }
        Ok(())
    }

    pub fn get_match_relevance(&self, articles: &[ArxivEntry]) -> Vec<(usize, bool)> {
        let query_lower = self.query.to_lowercase();
        self.filtered_indices
            .iter()
            .map(|&idx| {
                let haystack =
                    format!("{} {}", articles[idx].title, articles[idx].summary).to_lowercase();
                (idx, haystack.contains(&query_lower))
            })
            .collect()
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SearchState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SearchState")
            .field("query", &self.query)
            .field("results_count", &self.filtered_indices.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock helper since we don't want to rely on arxiv fetching logic here
    fn create_mock_articles() -> Vec<ArxivEntry> {
        vec![
            ArxivEntry::new(
                "Rust Programming".into(),
                vec![],
                "Safety and speed".into(),
                "1".into(),
                "2026".into(),
                "2026".into(),
            ),
            ArxivEntry::new(
                "Python Data".into(),
                vec![],
                "Easy and slow".into(),
                "2".into(),
                "2026".into(),
                "2026".into(),
            ),
        ]
    }

    #[test]
    fn test_state_caching_logic() {
        let mut state = SearchState::new();
        let articles = create_mock_articles();

        // Initially empty
        assert_eq!(state.filtered_count(), 0);

        // Load articles
        state.set_articles(&articles);

        // With empty query, should show all
        assert_eq!(state.filtered_count(), 2);

        // Update query
        state.update_query("Rust".to_string());
        assert_eq!(state.filtered_count(), 1);
        assert_eq!(state.filtered_indices[0], 0); // Matches "Rust Programming"
    }

    #[test]
    fn test_incremental_search() {
        let mut state = SearchState::new();
        state.set_articles(&create_mock_articles());

        state.push_char('P');
        state.push_char('y');

        assert_eq!(state.query, "Py");
        assert_eq!(state.filtered_count(), 1);
        assert_eq!(state.filtered_indices[0], 1); // Matches "Python Data"

        state.pop_char();
        assert_eq!(state.query, "P");
        // "P" matches both "Programming" and "Python"
        assert_eq!(state.filtered_count(), 2);
    }

    #[test]
    fn test_clear_resets_state_but_keeps_haystack() {
        let mut state = SearchState::new();
        state.set_articles(&create_mock_articles());
        state.update_query("Rust".to_string());

        state.clear();

        assert_eq!(state.query, "");
        assert_eq!(state.filtered_count(), 2);
        assert!(
            !state.haystacks.is_empty(),
            "Should not delete processed data on clear"
        );
    }
    #[test]
    fn test_search_connectivity_to_engine() {
        let mut state = SearchState::new();
        let articles = vec![ArxivEntry::new(
            "Quantum Computing".into(),
            vec!["Alice".into()],
            "A deep dive into qubits.".into(),
            "http://arxiv.org/abs/1".into(),
            "2026-01-01".into(),
            "2026-01-01".into(),
        )];

        // Step 1: Set articles (This MUST generate haystacks)
        state.set_articles(&articles);

        // Step 2: Search for a word known to be in the title
        state.update_query("Quantum".to_string());

        // If this fails, the Engine or Haystacks are the problem
        assert_eq!(
            state.filtered_indices.len(),
            1,
            "Should find 'Quantum' in title"
        );

        // Step 3: Search for a word known to be in the summary
        state.update_query("qubits".to_string());
        assert_eq!(
            state.filtered_indices.len(),
            1,
            "Should find 'qubits' in summary"
        );
    }
}
