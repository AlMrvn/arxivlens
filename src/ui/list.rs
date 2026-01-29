use crate::arxiv::ArxivQueryResult;
use crate::ui::utils::check_author_match;
use crate::ui::Theme;
use nucleo::pattern::{CaseMatching, Normalization, Pattern};
use nucleo::Matcher;
use ratatui::widgets::{List, ListState};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, HighlightSpacing, ListDirection, ListItem},
    Frame,
};

#[derive(Debug)]
pub struct ArticleFeed<'a> {
    items: List<'a>,
    pub state: ListState,
    pub len: usize,
}

impl ArticleFeed<'_> {
    pub fn new(
        query_result: &ArxivQueryResult,
        highlight_authors: Option<&[&str]>,
        theme: &Theme,
    ) -> Self {
        Self::new_with_search(query_result, highlight_authors, theme, None)
    }

    pub fn new_with_search(
        query_result: &ArxivQueryResult,
        highlight_authors: Option<&[&str]>,
        theme: &Theme,
        search_state: Option<&crate::app::search::SearchState>,
    ) -> Self {
        // 1. Determine which indices we are actually showing
        let filtered_indices: Vec<usize> = if let Some(search) = search_state {
            // Always use the filtered_indices from search_state, whether active or not
            // When search is not active, filtered_indices should contain all indices
            search.filtered_indices.clone()
        } else {
            (0..query_result.articles.len()).collect()
        };

        let entry_count = filtered_indices.len();

        // 2. Iterate ONLY over the filtered indices
        let items: Vec<ListItem> = filtered_indices
            .iter()
            .map(|&index| {
                let entry = &query_result.articles[index];

                let base_style = if highlight_authors
                    .is_some_and(|patterns| check_author_match(&entry.authors, patterns))
                {
                    theme.title
                } else {
                    theme.main
                };

                // 3. Apply highlighting (using our engine-backed search_state)
                let title_line = if let Some(search) = search_state {
                    if search.is_active() {
                        Self::create_fuzzy_highlighted_line(
                            &entry.title,
                            index,
                            search,
                            theme,
                            base_style,
                        )
                    } else {
                        Line::from(entry.title.clone()).style(base_style)
                    }
                } else {
                    Line::from(entry.title.clone()).style(base_style)
                };

                ListItem::from(title_line)
            })
            .collect();

        // ... (The rest of the List configuration stays the same) ...
        let highlight_style = if search_state.is_some_and(|s| s.is_active()) {
            Style::default()
        } else {
            theme.selection
        };

        let items = List::new(items)
            .block(
                Block::bordered()
                    .title_style(theme.title)
                    .title_alignment(Alignment::Left)
                    .title("arXiv Feed"),
            )
            .style(theme.main)
            .highlight_style(highlight_style)
            .highlight_symbol("> ")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom)
            .highlight_spacing(HighlightSpacing::Always);

        Self {
            items,
            state: ListState::default(),
            len: entry_count,
        }
    }

    fn create_fuzzy_highlighted_line(
        text: &str,
        _article_index: usize,
        search_state: &crate::app::search::SearchState,
        theme: &Theme,
        base_style: Style,
    ) -> Line<'static> {
        // 1. If no query, return the whole line with the base style
        if search_state.query.is_empty() {
            return Line::from(text.to_string()).style(base_style);
        }

        // 2. Setup the Nucleo Pattern and Matcher
        let pattern = Pattern::parse(
            &search_state.query,
            CaseMatching::Ignore,
            Normalization::Smart,
        );

        let mut matcher = Matcher::new(nucleo::Config::DEFAULT);
        let mut indices = Vec::new();

        // 3. Convert text to Utf32String and get a view using .slice(..)
        let haystack_buffer = nucleo::Utf32String::from(text);
        let haystack = haystack_buffer.slice(..);

        // 4. Run the matching engine to find character positions
        if pattern
            .indices(haystack, &mut matcher, &mut indices)
            .is_some()
        {
            let mut spans = Vec::new();
            let chars: Vec<char> = text.chars().collect();
            let mut current_span_start = 0;
            let mut current_style = base_style;

            // Create the highlight style based on your Tokyo Night ORANGE
            // We use .patch so it looks good even when the row is selected
            let highlight_style = base_style
                .patch(theme.highlight)
                .add_modifier(Modifier::BOLD);

            for (char_index, _ch) in chars.iter().enumerate() {
                let should_highlight = indices.contains(&(char_index as u32));
                let new_style = if should_highlight {
                    highlight_style
                } else {
                    base_style
                };

                // If the style changes (start/end of a highlight), push the previous span
                if new_style != current_style {
                    if char_index > current_span_start {
                        let span_text: String =
                            chars[current_span_start..char_index].iter().collect();
                        spans.push(Span::styled(span_text, current_style));
                    }
                    current_span_start = char_index;
                    current_style = new_style;
                }
            }

            // Push the remaining part of the string
            if current_span_start < chars.len() {
                let span_text: String = chars[current_span_start..].iter().collect();
                spans.push(Span::styled(span_text, current_style));
            }

            Line::from(spans)
        } else {
            // Fallback: if search is active but this specific string didn't match
            Line::from(text.to_string()).style(base_style)
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(&self.items, area, &mut self.state);
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod feed_tests {
    use super::*;
    use crate::app::search::SearchState;
    use crate::arxiv::{ArxivEntry, ArxivQueryResult};
    use crate::ui::Theme;

    #[test]
    fn test_feed_constructor_filters_correctly() {
        let articles = vec![
            ArxivEntry::new(
                "Rust".into(),
                vec![],
                "Summary".into(),
                "1".into(),
                "2026".into(),
                "2026".into(),
            ),
            ArxivEntry::new(
                "Python".into(),
                vec![],
                "Summary".into(),
                "2".into(),
                "2026".into(),
                "2026".into(),
            ),
        ];
        let query_result = ArxivQueryResult {
            updated: "".into(),
            articles,
        };
        let theme = Theme::default();
        let mut search_state = SearchState::new();
        search_state.set_articles(&query_result.articles);
        search_state.update_query("Rust".to_string());
        // This is what we are testing:
        let feed = ArticleFeed::new_with_search(&query_result, None, &theme, Some(&search_state));
        assert_eq!(
            feed.len, 1,
            "Feed length should be 1 after filtering for 'Rust'"
        );
    }
}
