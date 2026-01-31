use nucleo::pattern::{CaseMatching, Normalization, Pattern};
use nucleo::{Config, Nucleo};
use std::fmt;
use std::sync::Arc;

/// Configuration for the search engine scoring and filtering behavior.
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Minimum score threshold for including results in the final output.
    /// Results with scores below this value are filtered out to reduce noise.
    pub strictness_threshold: u32,

    /// Score boost for exact substring matches of the entire query.
    /// This is the highest priority match type.
    pub match_boost_exact_substring: u32,

    /// Additional score boost when the exact substring appears at the beginning of the text.
    /// Applied on top of exact substring boost for prefix matches.
    pub match_boost_prefix: u32,

    /// Additional score boost when the query matches at word boundaries.
    /// Applied when any word in the text starts with the query string.
    pub match_boost_word_boundary: u32,

    /// Additional score boost for exact word matches.
    /// Applied when the query exactly matches a complete word in the text.
    pub match_boost_exact_word: u32,

    /// Score boost when all search words are present as substrings.
    /// Used for multi-word queries where each word must be found somewhere in the text.
    pub match_boost_all_words_present: u32,

    /// Score boost for fuzzy matches using character windows.
    /// This is the lowest priority match type, used only as a fallback.
    pub match_boost_fuzzy_window: u32,

    /// Minimum word length to include in search filtering.
    /// Words shorter than this are ignored during multi-word search processing.
    pub min_word_length_for_filter: usize,

    /// Size of character windows used for fuzzy matching.
    /// Larger windows are more permissive but may allow false positives.
    pub fuzzy_window_size: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            strictness_threshold: 200,
            match_boost_exact_substring: 1000,
            match_boost_prefix: 500,
            match_boost_word_boundary: 300,
            match_boost_exact_word: 200,
            match_boost_all_words_present: 250,
            match_boost_fuzzy_window: 50,
            min_word_length_for_filter: 3,
            fuzzy_window_size: 4,
        }
    }
}

/// A domain-agnostic fuzzy search engine with strict word-matching guardrails.
pub struct SearchEngine {
    matcher: Nucleo<usize>,
    config: SearchConfig,
}

impl SearchEngine {
    /// Creates a new search engine with default configuration.
    pub fn new() -> Self {
        Self::with_config(SearchConfig::default())
    }

    /// Creates a new search engine with custom configuration.
    pub fn with_config(config: SearchConfig) -> Self {
        Self {
            matcher: Nucleo::new(Config::DEFAULT, Arc::new(|| {}), None, 1),
            config,
        }
    }

    /// Filters a list of strings and returns the indices of those that match.
    /// # Arguments
    /// * `query` - The search string from the user.
    /// * `haystacks` - A slice of strings to search through (e.g., pre-rendered article text).
    pub fn filter(&mut self, query: &str, haystacks: &[String]) -> Vec<usize> {
        if query.is_empty() {
            return (0..haystacks.len()).collect();
        }

        // 1. Reset and Inject
        self.matcher.restart(false);
        let injector = self.matcher.injector();
        for (index, text) in haystacks.iter().enumerate() {
            injector.push(index, |_, dst| {
                dst[0] = text.as_str().into();
            });
        }

        // 2. Configure Pattern
        self.matcher
            .pattern
            .reparse(0, query, CaseMatching::Ignore, Normalization::Smart, true);

        // 3. Process matches
        self.matcher.tick(50);
        let snapshot = self.matcher.snapshot();
        let query_lower = query.to_lowercase();

        // Handle hyphenated terms as single tokens first, then split by whitespace
        let normalized_query = query_lower.replace('-', " ");
        let search_words: Vec<&str> = normalized_query
            .split_whitespace()
            .filter(|w| w.len() >= self.config.min_word_length_for_filter)
            .collect();

        // 4. Manual Scoring and Filtering with substring priority
        let mut scored_matches = Vec::new();

        // Iterate through matches provided by nucleo
        for i in 0..snapshot.matched_item_count() {
            if let Some(item) = snapshot.get_matched_item(i) {
                let index = *item.data;
                let content = haystacks[index].to_lowercase();

                // Calculate substring-prioritized score
                let mut score: u32 = 0;

                // HIGHEST PRIORITY: Exact substring match of entire query
                if content.contains(&query_lower) {
                    score += self.config.match_boost_exact_substring;

                    // Bonus for prefix match
                    if content.starts_with(&query_lower) {
                        score += self.config.match_boost_prefix;
                    }

                    // Bonus for word boundary match
                    if content
                        .split_whitespace()
                        .any(|w| w.starts_with(&query_lower))
                    {
                        score += self.config.match_boost_word_boundary;
                    }

                    // Bonus for exact word match
                    if content.split_whitespace().any(|w| w == query_lower) {
                        score += self.config.match_boost_exact_word;
                    }
                } else if !search_words.is_empty() {
                    // MEDIUM PRIORITY: Check if all search words are present as substrings
                    let all_words_present = search_words.iter().all(|&word| content.contains(word));

                    if all_words_present {
                        score += self.config.match_boost_all_words_present;
                    } else if query_lower.len() >= self.config.fuzzy_window_size {
                        // LOWEST PRIORITY: Fuzzy matching with configurable windows (only for longer queries)
                        let fuzzy_match = search_words.iter().all(|&word| {
                            if content.contains(word) {
                                return true;
                            }
                            word.as_bytes()
                                .windows(self.config.fuzzy_window_size)
                                .any(|w| {
                                    std::str::from_utf8(w)
                                        .map(|s| content.contains(s))
                                        .unwrap_or(false)
                                })
                        });

                        if fuzzy_match {
                            score += self.config.strictness_threshold;
                        }
                    }
                }

                // Apply strictness threshold - only include high-quality matches
                if score >= self.config.strictness_threshold {
                    // Favor shorter strings for tie-breaking
                    score += (100usize.saturating_sub(content.len().min(100))) as u32;
                    scored_matches.push((index, score));
                }
            }
        }

        // 5. Sort by our calculated boost score
        scored_matches.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        scored_matches.into_iter().map(|(idx, _)| idx).collect()
    }

    /// Generates highlighting indices for a specific string based on the current query.
    /// This uses the internal matcher to ensure UI highlighting matches search logic.
    pub fn get_highlight_indices(&mut self, query: &str, text: &str) -> Vec<u32> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut indices = Vec::new();
        let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
        let haystack = nucleo::Utf32String::from(text);

        // Use a lightweight matcher for individual string highlighting
        let mut matcher = nucleo::Matcher::new(nucleo::Config::DEFAULT);

        if pattern
            .indices(haystack.slice(..), &mut matcher, &mut indices)
            .is_some()
        {
            indices
        } else {
            Vec::new()
        }
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Manual implementation since Nucleo doesn't support Debug
impl fmt::Debug for SearchEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SearchEngine")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agnostic_and_logic() {
        let mut engine = SearchEngine::new();
        let data = vec![
            "Quantum Computing".to_string(),
            "Machine Learning".to_string(),
            "Quantum Machine Learning".to_string(),
        ];

        let results = engine.filter("Quantum Machine", &data);

        // Should only return the index of the string containing BOTH words
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 2);
    }

    #[test]
    fn test_typo_tolerance_window() {
        let mut engine = SearchEngine::new();
        let data = vec!["Computational Biology".to_string()];

        // "Computatinal" (typo) still has 4-char overlaps like "Comp", "mput", etc.
        let results = engine.filter("Computatinal", &data);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_guardrail_prevents_false_fuzzy_match() {
        let mut engine = SearchEngine::new();
        let data = vec!["Linear Algebra".to_string()];

        // Nucleo might fuzzy match "Learner" to "Linear", but our guardrail
        // will check if "Learner" or its windows exist. They don't.
        let results = engine.filter("Learner", &data);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_empty_query_returns_everything() {
        let mut engine = SearchEngine::new();
        let data = vec!["A".into(), "B".into(), "C".into()];
        let results = engine.filter("", &data);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_short_words_passthrough() {
        let mut engine = SearchEngine::new();
        let data = vec!["The Art of War".into()];

        // Words < 4 chars are handled by Nucleo's default fuzzy logic
        // without the extra strict guardrail check.
        let results = engine.filter("Art", &data);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_short_incremental_match() {
        let mut engine = SearchEngine::new();
        let data = vec![
            "Programming".to_string(),
            "Python Data".to_string(),
            "Rust".to_string(),
        ];

        // "Py" should match "Python Data" but NOT "Programming"
        // if we want strict prefix/substring behavior.
        let results = engine.filter("Py", &data);

        // If this returns [0, 1], your guardrail is too loose for short words.
        // If it returns [1], it's working as intended for incremental typing.
        assert_eq!(results, vec![1]);
    }

    #[test]
    fn test_long_incremental_match() {
        let mut engine = SearchEngine::new();
        let data = vec![
            "Deep Learning for Vision".to_string(),
            "Deep Reinforcement Learning".to_string(),
            "Machine Learning Basics".to_string(),
        ];

        // Stage 1: "Deep"
        let res1 = engine.filter("Deep", &data);
        assert_eq!(res1.len(), 2); // Matches first two

        // Stage 2: "Deep Learn"
        let res2 = engine.filter("Deep Learn", &data);
        assert_eq!(res2.len(), 2); // Still matches first two

        // Stage 3: "Deep Learning Vi"
        let res3 = engine.filter("Deep Learning Vi", &data);
        assert_eq!(res3.len(), 1);
        assert_eq!(res3[0], 0); // Only matches "Deep Learning for Vision"
    }

    #[test]
    fn test_short_query_strict_substring() {
        let mut engine = SearchEngine::new();
        let data = vec![
            "Programming".to_string(),
            "Python".to_string(),
            "Physics".to_string(),
        ];

        // Under the strict < 3 char rule, "Py" should ONLY match "Python"
        let results = engine.filter("Py", &data);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 1); // Index of "Python"
    }

    #[test]
    fn test_custom_config_impact() {
        // Create a config with very high strictness threshold
        let strict_config = SearchConfig {
            strictness_threshold: 2000, // Much higher than any normal score
            ..SearchConfig::default()
        };

        let mut strict_engine = SearchEngine::with_config(strict_config);
        let data = vec!["Machine Learning".to_string(), "Deep Learning".to_string()];

        // With high strictness, even good matches should be filtered out
        let results = strict_engine.filter("Learning", &data);
        assert_eq!(
            results.len(),
            0,
            "High strictness threshold should filter out all results"
        );

        // Compare with default engine
        let mut default_engine = SearchEngine::new();
        let default_results = default_engine.filter("Learning", &data);
        assert!(
            default_results.len() > 0,
            "Default engine should find matches"
        );
    }
}
