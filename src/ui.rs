use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, HighlightSpacing, List, ListDirection, ListItem, Paragraph, Wrap,
    },
    Frame,
};

use crate::app::App;

fn split_with_keywords(text: &str, keywords: &[&str]) -> (Vec<String>, Vec<bool>) {
    let mut text_chunks = vec![];
    let mut is_keyword = vec![];

    for word in text.split_whitespace() {
        if keywords.contains(&word) {
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
        let keywords = ["alpha", "beta"];
        let (result, is_keyword) = split_with_keywords(&text, &keywords);

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
        .constraints([Constraint::Length(6), Constraint::Min(10)])
        .split(layout[1]);

    // Create the block:
    let block = Block::bordered()
        .title("arXiv Read")
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
        .style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
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
    let authors = if let Some(i) = app.arxiv_entries.state.selected() {
        app.arxiv_entries.items[i].authors.join(", ")
    } else {
        "Nothing selected...".to_string()
    };

    frame.render_widget(
        Paragraph::new(format!("{}", authors))
            .block(
                Block::bordered()
                    .title("Authors")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::Cyan).bg(Color::Black))
            .left_aligned()
            .wrap(Wrap { trim: true }),
        sub_layout[0],
    );

    // The abstract of the manuscript
    let summary = if let Some(i) = app.arxiv_entries.state.selected() {
        app.arxiv_entries.items[i].summary.clone()
    } else {
        "Nothing selected...".to_string()
    };

    let mut spans: Vec<Span> = Vec::new();
    let (splitted_summary, is_keyword) =
        split_with_keywords(&summary, &["error", "correction", "Correction"]);
    for (chunk, is_key) in splitted_summary.iter().zip(is_keyword.iter()) {
        if *is_key {
            spans.push(Span::raw(chunk).style(Style::default().fg(Color::Black).bg(Color::White)));
        } else {
            spans.push(Span::raw(chunk).style(Style::default().fg(Color::Cyan).bg(Color::Black)));
        }
    }
    let text = Text::from(Line::from(spans));

    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::bordered()
                    .title("Abstract")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::Cyan).bg(Color::Black))
            .left_aligned()
            .wrap(Wrap { trim: true }),
        sub_layout[1],
    )
}
