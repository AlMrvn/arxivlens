use crate::ui::theme::Theme;
use ratatui::text::{Line, Span};

/// Handles the conversion of text and match indices into a styled Ratatui Line.
pub struct Highlighter;

impl Highlighter {
    /// Takes a raw string and a set of character indices (from SearchEngine)
    /// and returns a styled Line using the provided theme.
    pub fn fuzzy_line<'a>(text: &'a str, indices: &[u32], theme: &Theme) -> Line<'a> {
        if indices.is_empty() {
            return Line::from(Span::styled(text, theme.main));
        }

        let mut spans = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut current_chunk = String::new();
        let mut last_was_highlighted = false;

        for (i, &ch) in chars.iter().enumerate() {
            let is_highlighted = indices.contains(&(i as u32));

            // If the status changes (e.g. from normal to highlighted),
            // push the accumulated chunk and start a new one.
            if is_highlighted != last_was_highlighted && !current_chunk.is_empty() {
                let style = if last_was_highlighted {
                    theme.highlight
                } else {
                    theme.main
                };
                spans.push(Span::styled(current_chunk.clone(), style));
                current_chunk.clear();
            }

            current_chunk.push(ch);
            last_was_highlighted = is_highlighted;
        }

        // Push the final chunk
        if !current_chunk.is_empty() {
            let style = if last_was_highlighted {
                theme.highlight
            } else {
                theme.main
            };
            spans.push(Span::styled(current_chunk, style));
        }

        Line::from(spans)
    }
}
