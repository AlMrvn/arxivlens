//! Module for highligting keyword in a text.

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use aho_corasick::AhoCorasick;

const TURQUOISE: Color = Color::Rgb(79, 214, 190);
const TEAL: Color = Color::Rgb(65, 166, 181);
const SEARCH_HL_STYLE: Style = Style::new().fg(Color::Black).bg(TURQUOISE);
const MAIN_STYLE: Style = Style::new().fg(TEAL).bg(Color::Black);

fn search_patterns(text: &str, patterns: &[&str]) -> Vec<(usize, usize)> {
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
pub fn highlight_patterns<'a>(text: &'a str, patterns: Option<&[&str]>) -> Line<'a> {
    let patterns = patterns.unwrap_or_default();
    let match_locs = search_patterns(text, patterns);

    if match_locs.len() == 0 {
        Line::from(Span::raw(text).style(MAIN_STYLE))
    } else {
        let mut start_chunk: usize = 0;
        let mut hilighted_spans: Vec<Span> = Vec::new();
        for (start, end) in match_locs.iter() {
            hilighted_spans.push(Span::raw(&text[start_chunk..*start]).style(MAIN_STYLE));
            hilighted_spans.push(Span::raw(&text[*start..*end]).style(SEARCH_HL_STYLE));
            start_chunk = *end;
        }
        Line::from(hilighted_spans)
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
        let text = "This is a text with some keywords like hello and world";
        let patterns = &["hello", "world"];

        let expected_spans = vec![
            Span::raw("This is a text with some keywords like ").style(MAIN_STYLE),
            Span::raw("hello").style(SEARCH_HL_STYLE),
            Span::raw(" and ").style(MAIN_STYLE),
            Span::raw("world").style(SEARCH_HL_STYLE),
        ];

        let result = highlight_patterns(text, Some(patterns));

        assert_eq!(result.spans, expected_spans);
    }

    #[test]
    fn test_highlight_patterns_no_match() {
        let text = "This is a text without any keywords";
        let patterns = &["hello", "world"];

        let expected_spans = vec![Span::raw(text).style(MAIN_STYLE)];

        let result = highlight_patterns(text, Some(patterns));

        assert_eq!(result.spans, expected_spans);
    }

    #[test]
    fn test_highlight_patterns_none() {
        let text = "This is a text without any keywords";

        let expected_spans = vec![Span::raw(text).style(MAIN_STYLE)];

        let result = highlight_patterns(text, None);

        assert_eq!(result.spans, expected_spans);
    }
}