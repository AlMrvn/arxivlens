use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
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
const HIGHLIGHT_STYLE: Style = Style::new()
    .fg(TEAL)
    .bg(Color::White)
    .add_modifier(Modifier::ITALIC);
const MAIN_STYLE: Style = Style::new().fg(TEAL).bg(Color::Black);

// Create the block:
fn get_template_block() -> Block<'static> {
    Block::new()
        .borders(Borders::TOP)
        .title_style(Style::new().fg(ORANGE))
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Plain)
        .padding(Padding::horizontal(2))
}

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

/// Renders the arXiv feed with a selection
fn render_feed(app: &mut App, frame: &mut Frame, area: Rect) {
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
        .block(get_template_block().title("arXiv Feed"))
        .style(MAIN_STYLE)
        .highlight_style(HIGHLIGHT_STYLE)
        .highlight_symbol("> ")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom)
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(list, area, &mut app.arxiv_entries.state);
}

fn render_selected_entry(app: &mut App, frame: &mut Frame, area: Rect) {
    // first split the area
    let sub_layout = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(2)
        .constraints([
            Constraint::Length(4), // Title
            Constraint::Length(6), // Authors
            Constraint::Min(10),   // Abstract/summary
        ])
        .split(area);

    // Authors of the manuscript:
    let current_entry = if let Some(i) = app.arxiv_entries.state.selected() {
        &app.arxiv_entries.items[i]
    } else {
        // Should implement a default print here ?
        &app.arxiv_entries.items[0]
    };

    // Title
    frame.render_widget(
        Paragraph::new(Line::raw(current_entry.title.clone()))
            .block(get_template_block().title(" Title "))
            .style(MAIN_STYLE)
            .left_aligned()
            .wrap(Wrap { trim: true }),
        sub_layout[0],
    );

    // Authors
    frame.render_widget(
        Paragraph::new(format!("{}", current_entry.authors.join(", ")))
            .block(get_template_block().title(" Author "))
            .style(MAIN_STYLE)
            .left_aligned()
            .wrap(Wrap { trim: true }),
        sub_layout[1],
    );

    // Implementation of the highlight of keywords:
    let mut spans: Vec<Span> = Vec::new();
    if let Some(highlight) = &app.summary_highlight {
        let (splitted_summary, is_keyword) =
            split_with_keywords(current_entry.summary.clone(), highlight.to_vec());
        for (chunk, is_key) in splitted_summary.iter().zip(is_keyword.iter()) {
            if *is_key {
                spans.push(
                    Span::raw(chunk.clone()).style(Style::default().fg(Color::Black).bg(TURQUOISE)),
                );
            } else {
                spans.push(Span::raw(chunk.clone()).style(MAIN_STYLE));
            }
        }
    } else {
        spans.push(Span::raw(&current_entry.summary).style(MAIN_STYLE));
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .block(get_template_block().title(" Abstract "))
            .style(MAIN_STYLE)
            .left_aligned()
            .wrap(Wrap { trim: true }),
        sub_layout[2],
    )
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

    // Render the slectable feed
    render_feed(app, frame, layout[0]);

    // Render the right panel:
    frame.render_widget(
        Paragraph::new("").block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        ),
        layout[1],
    );

    render_selected_entry(app, frame, layout[1])
}
