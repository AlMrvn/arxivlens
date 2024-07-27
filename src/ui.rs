use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{
        Block, BorderType, Borders, HighlightSpacing, List, ListDirection, ListItem, Padding,
        Paragraph, Wrap,
    },
    Frame,
};

use crate::app::App;
use crate::search_highlight::highlight_patterns;

// Using the Tokyonight color palette. See https://lospec.com/palette-list/tokyo-night.
const ORANGE: Color = Color::Rgb(255, 158, 100);
const TURQUOISE: Color = Color::Rgb(79, 214, 190);
const TEAL: Color = Color::Rgb(65, 166, 181);
const HIGHLIGHT_STYLE: Style = Style::new()
    .fg(TEAL)
    .bg(Color::White)
    .add_modifier(Modifier::ITALIC);
const SEARCH_HL_STYLE: Style = Style::new().fg(Color::Black).bg(TURQUOISE);
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

fn render_entry_with_pattern_highlight(
    title: &str,
    entry: &str,
    patterns: Option<&[&str]>,
    frame: &mut Frame,
    area: Rect,
) {
    frame.render_widget(
        Paragraph::new(highlight_patterns(entry, patterns))
            .block(get_template_block().title(title))
            .style(MAIN_STYLE)
            .left_aligned()
            .wrap(Wrap { trim: true }),
        area,
    )
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
    render_entry_with_pattern_highlight(
        " Title ",
        &current_entry.title,
        None,
        frame,
        sub_layout[0],
    );

    // Authors
    render_entry_with_pattern_highlight(
        " Author ",
        &format!("{}", current_entry.authors.join(", ")),
        None,
        frame,
        sub_layout[1],
    );

    // Implementation of the highlight of keywords:
    render_entry_with_pattern_highlight(
        " Abstract ",
        &current_entry.summary,
        Some(
            &app.summary_highlight
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
        ),
        frame,
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
