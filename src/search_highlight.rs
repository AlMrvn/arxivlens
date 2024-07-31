//! Module for highligting keyword in a text.

use ratatui::text::{Line, Span};

use aho_corasick::AhoCorasick;

use crate::ui::Theme;

pub fn search_patterns(text: &str, patterns: &[&str]) -> Vec<(usize, usize)> {
    let ac = AhoCorasick::builder()
        .ascii_case_insensitive(true)
        .build(patterns)
        .unwrap();
    let mut matches = vec![];
    for mat in ac.find_iter(text) {
        matches.push((mat.start(), mat.end()));
    }
    matches
}

/// Highligh the pattern matched.
///
/// The lifetime of the output is only due to the lifetime of the text, not of the
/// patterns.
pub fn highlight_patterns<'a>(text: &'a str, patterns: Option<&[&str]>, theme: &Theme) -> Line<'a> {
    let patterns = patterns.unwrap_or_default();
    let match_locs = search_patterns(text, patterns);

    if match_locs.len() == 0 {
        Line::from(Span::raw(text).style(theme.main))
    } else {
        let mut start_chunk: usize = 0;
        let mut highlighted_spans: Vec<Span> = Vec::new();
        for (start, end) in match_locs.iter() {
            highlighted_spans.push(Span::raw(&text[start_chunk..*start]).style(theme.main));
            highlighted_spans.push(Span::raw(&text[*start..*end]).style(theme.highlight));
            start_chunk = *end;
        }

        // Adding the last bit if necessary:
        if start_chunk != text.len() {
            highlighted_spans.push(Span::raw(&text[start_chunk..]).style(theme.main));
        }
        Line::from(highlighted_spans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- Testing the search patterns function. Example from AhoCorasick docs -----
    #[test]
    fn test_search_patterns() {
        let patterns = &["apple", "maple", "Snapple"];
        let text = "Nobody likes maple in their apple flavored Snapple.";

        let match_locs = search_patterns(text, patterns);

        assert_eq!(match_locs, vec![(13, 18), (28, 33), (43, 50),]);
    }

    #[test]
    fn test_highlight_patterns() {
        let theme = Theme::default();
        let text = "This is a text with some keywords like hello and world";
        let patterns = &["hello", "world"];

        let expected_spans = vec![
            Span::raw("This is a text with some keywords like ").style(theme.main),
            Span::raw("hello").style(theme.highlight),
            Span::raw(" and ").style(theme.main),
            Span::raw("world").style(theme.highlight),
        ];

        let result = highlight_patterns(text, Some(patterns), &theme);

        assert_eq!(result.spans, expected_spans);
    }

    #[test]
    fn test_highlight_patterns_no_match() {
        let theme = Theme::default();
        let text = "This is a text without any keywords";
        let patterns = &["hello", "world"];

        let expected_spans = vec![Span::raw(text).style(theme.main)];

        let result = highlight_patterns(text, Some(patterns), &theme);

        assert_eq!(result.spans, expected_spans);
    }

    #[test]
    fn test_highlight_patterns_none() {
        let theme = Theme::default();
        let text = "This is a text without any keywords";

        let expected_spans = vec![Span::raw(text).style(theme.main)];

        let result = highlight_patterns(text, None, &theme);

        assert_eq!(result.spans, expected_spans);
    }

    #[test]
    fn test_highlight_pattern_end_of_text() {
        let theme = Theme::default();
        let text = "This is a text with some keywords like hello and world";
        let patterns = &["hello"];

        let expected_spans = vec![
            Span::raw("This is a text with some keywords like ").style(theme.main),
            Span::raw("hello").style(theme.highlight),
            Span::raw(" and world").style(theme.main),
        ];

        let result = highlight_patterns(text, Some(patterns), &theme);

        assert_eq!(result.spans, expected_spans);
    }
}
