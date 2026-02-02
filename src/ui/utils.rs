//! Module for highlighting keyword in a text.

use aho_corasick::AhoCorasick;
use ratatui::text::{Line, Span};

use crate::ui::Theme;

/// Convert Option<Vec<String>> to Option<Vec<&str>>
pub fn option_vec_to_option_slice(option_vec: &Option<Vec<String>>) -> Option<Vec<&str>> {
    option_vec
        .as_deref()
        .map(|v| v.iter().map(String::as_str).collect::<Vec<&str>>())
}

pub fn check_author_match(authors: &[String], patterns: &[&str]) -> bool {
    if patterns.is_empty() {
        return false;
    }

    let author_text = authors.join(", ");
    let matches = search_patterns(&author_text, patterns);
    !matches.is_empty()
}

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
pub fn highlight_patterns<'a>(
    text: &'a str,
    patterns: Option<&[&str]>,
    theme: &Theme,
    base_style: ratatui::style::Style,
) -> Line<'a> {
    let patterns = patterns.unwrap_or_default();
    let match_locs = search_patterns(text, patterns);

    if match_locs.is_empty() {
        Line::from(Span::raw(text).style(base_style)) // Use base_style
    } else {
        let mut start_chunk: usize = 0;
        let mut highlighted_spans: Vec<Span> = Vec::new();
        for (start, end) in match_locs.iter() {
            // Use base_style for the non-matched parts
            highlighted_spans.push(Span::raw(&text[start_chunk..*start]).style(base_style));
            // Use the specific highlighted style for matches
            highlighted_spans.push(Span::raw(&text[*start..*end]).style(theme.list.highlighted));
            start_chunk = *end;
        }

        if start_chunk != text.len() {
            highlighted_spans.push(Span::raw(&text[start_chunk..]).style(base_style));
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
        let base_style = theme.main;

        let expected_spans = vec![
            Span::raw("This is a text with some keywords like ").style(base_style),
            Span::raw("hello").style(theme.list.highlighted),
            Span::raw(" and ").style(base_style),
            Span::raw("world").style(theme.list.highlighted),
        ];

        let result = highlight_patterns(text, Some(patterns), &theme, base_style);

        assert_eq!(result.spans, expected_spans);
    }

    #[test]
    fn test_highlight_patterns_no_match() {
        let theme = Theme::default();
        let text = "This is a text without any keywords";
        let patterns = &["hello", "world"];
        let base_style = theme.main;

        let expected_spans = vec![Span::raw(text).style(base_style)];

        let result = highlight_patterns(text, Some(patterns), &theme, base_style);

        assert_eq!(result.spans, expected_spans);
    }

    #[test]
    fn test_highlight_patterns_none() {
        let theme = Theme::default();
        let text = "This is a text without any keywords";
        let base_style = theme.main;

        let expected_spans = vec![Span::raw(text).style(base_style)];

        let result = highlight_patterns(text, None, &theme, base_style);

        assert_eq!(result.spans, expected_spans);
    }

    #[test]
    fn test_highlight_pattern_end_of_text() {
        let theme = Theme::default();
        let text = "This is a text with some keywords like hello and world";
        let patterns = &["hello"];
        let base_style = theme.main;

        let expected_spans = vec![
            Span::raw("This is a text with some keywords like ").style(base_style),
            Span::raw("hello").style(theme.list.highlighted),
            Span::raw(" and world").style(base_style),
        ];

        let result = highlight_patterns(text, Some(patterns), &theme, base_style);

        assert_eq!(result.spans, expected_spans);
    }
}
