use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, HighlightSpacing, List, ListDirection, ListItem, Padding,
        Paragraph, Wrap,
    },
    Frame,
};

use crate::app::App;

// Using the Tokyonight color palette. See https://lospec.com/palette-list/tokyo-night.
const ORANGE: Color = Color::Rgb(255, 158, 100);
const TURQUOISE: Color = Color::Rgb(79, 214, 190);
const TEAL: Color = Color::Rgb(65, 166, 181);

fn split_with_keywords(text: String, keywords: Vec<String>) -> (Vec<String>, Vec<bool>) {
    let mut text_chunks = vec![];
    let mut is_keyword = vec![];

    for word in text.split_whitespace() {
        if keywords.contains(&word.to_string()) {
            text_chunks.push(word.to_string());
            is_keyword.push(true);
        } else {
            text_chunks.push(word.to_string());
            is_keyword.push(false);
        }
        text_chunks.push(" ".to_string());
        is_keyword.push(false);
    }

    (text_chunks, is_keyword)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_with_keywords() {
        let text = "This is a text with some keywords like alpha and beta";
        let keywords = ["alpha".to_string(), "beta".to_string()];
        let (result, is_keyword) = split_with_keywords(text.to_string(), keywords.to_vec());

        assert_eq!(
            result,
            vec![
                "This".to_string(),
                "is".to_string(),
                "a".to_string(),
                "text".to_string(),
                "with".to_string(),
                "some".to_string(),
                "keywords".to_string(),
                "like".to_string(),
                "alpha".to_string(),
                "and".to_string(),
                "beta".to_string(),
            ]
        );
        assert_eq!(
            is_keyword,
            vec![false, false, false, false, false, false, false, false, true, false, true]
        );
    }
}

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples

    // First we create a Layout
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(2)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.size());

    let sub_layout = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(2)
        .constraints([
            Constraint::Length(4), // Title
            Constraint::Length(6), // Authors
            Constraint::Min(10),   // Abstract/summery
        ])
        .split(layout[1]);

    // Create the block:
    let block = Block::bordered()
        .title("arXiv Read")
        .title_style(Style::default().fg(ORANGE))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);

    // Iterate through all elements in the `items` and use the title
    let items: Vec<ListItem> = app
        .arxiv_entries
        .items
        .iter()
        .enumerate()
        .map(|(_i, entry)| ListItem::from(entry.title.clone()))
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let list = List::new(items.clone())
        .block(block)
        .style(Style::default().fg(TEAL).bg(Color::Black))
        .highlight_style(
            Style::default()
                .fg(TEAL)
                .bg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )
        .highlight_symbol("> ")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom)
        .highlight_spacing(HighlightSpacing::Always);

    // The selection windows
    frame.render_stateful_widget(list, layout[0], &mut app.arxiv_entries.state);

    // Authors of the manuscript:
    let current_entry = if let Some(i) = app.arxiv_entries.state.selected() {
        &app.arxiv_entries.items[i]
    } else {
        // Should implement a default print here ?
        &app.arxiv_entries.items[0]
    };

    frame.render_widget(
        Paragraph::new("")
            .block(
                Block::bordered()
                    .title(" Abstract ")
                    .title_style(Style::default().fg(ORANGE))
                    .border_type(BorderType::Rounded)
                    .padding(Padding::vertical(2)),
            )
            .left_aligned()
            .wrap(Wrap { trim: true }),
        layout[1],
    );

    // Title
    frame.render_widget(
        Paragraph::new(Line::raw(current_entry.title.clone()))
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .title(" Title ")
                    .title_style(Style::default().fg(ORANGE))
                    .title_alignment(Alignment::Left)
                    .border_type(BorderType::Plain)
                    .padding(Padding::horizontal(2)),
            )
            .style(Style::default().fg(TEAL).bg(Color::Black))
            .left_aligned()
            .wrap(Wrap { trim: true }),
        sub_layout[0],
    );

    // Authors
    frame.render_widget(
        Paragraph::new(format!("{}", current_entry.authors.join(", ")))
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .title(" Authors ")
                    .title_style(Style::default().fg(ORANGE))
                    .title_alignment(Alignment::Left)
                    .border_type(BorderType::Double)
                    .padding(Padding::horizontal(2)),
            )
            .style(Style::default().fg(TEAL).bg(Color::Black))
            .left_aligned()
            .wrap(Wrap { trim: true }),
        sub_layout[1],
    );

    // The abstract of the manuscript.
    // Implementation of the hilight of keywords:
    let mut spans: Vec<Span> = Vec::new();
    if let Some(highlight) = &app.summary_highlight {
        let (splitted_summary, is_keyword) = split_with_keywords(
            current_entry.summary.clone(),
            // &["error", "correction", "Correction"],
            highlight.to_vec(),
        );
        for (chunk, is_key) in splitted_summary.iter().zip(is_keyword.iter()) {
            if *is_key {
                spans.push(
                    Span::raw(chunk.clone()).style(Style::default().fg(Color::Black).bg(TURQUOISE)),
                );
            } else {
                spans.push(
                    Span::raw(chunk.clone()).style(Style::default().fg(TEAL).bg(Color::Black)),
                );
            }
        }
    } else {
        spans.push(
            Span::raw(&current_entry.summary).style(Style::default().fg(TEAL).bg(Color::Black)),
        );
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .title(" Abstract ")
                    .title_style(Style::default().fg(ORANGE))
                    .title_alignment(Alignment::Left)
                    .border_type(BorderType::Plain)
                    .padding(Padding::horizontal(2)),
            )
            .style(Style::default().fg(TEAL).bg(Color::Black))
            .left_aligned()
            .wrap(Wrap { trim: true }),
        sub_layout[2],
    )
}
