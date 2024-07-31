use itertools::izip;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
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
const TEAL: Color = Color::Rgb(65, 166, 181);
const HIGHLIGHT_STYLE: Style = Style::new()
    .fg(TEAL)
    .bg(Color::White)
    .add_modifier(Modifier::ITALIC);
const MAIN_STYLE: Style = Style::new().fg(TEAL).bg(Color::Black);
const SHORTCUT_STYLE: Style = Style::new().fg(Color::Blue).bg(Color::Black);

fn option_vec_to_option_slice<'a>(option_vec: &'a Option<Vec<String>>) -> Option<Vec<&'a str>> {
    let binding = option_vec
        .as_deref()
        .map(|v| v.iter().map(String::as_str).collect::<Vec<&str>>());
    binding
}

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
    frame.render_stateful_widget(&app.listentry, area, &mut app.state);
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
    let current_entry = if let Some(i) = app.state.selected() {
        &app.query_result.articles[i]
    } else {
        // Should implement a default print here ?
        &app.query_result.articles[0]
    };

    // Zipping all the small info.
    let titles = vec![" Title ", " Author ", " Abstract "];
    let authors = format!("{}", current_entry.authors.join(", "));
    let entries = vec![&current_entry.title, &authors, &current_entry.summary];
    let patterns_list = vec![
        &app.highlight_config.keywords,
        &app.highlight_config.authors,
        &app.highlight_config.keywords,
    ];
    let areas = vec![sub_layout[0], sub_layout[1], sub_layout[2]];

    for (title, entry, patterns, area) in izip!(titles, entries, patterns_list, areas) {
        let patterns = option_vec_to_option_slice(patterns);
        render_entry_with_pattern_highlight(title, entry, patterns.as_deref(), frame, area)
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
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100), Constraint::Min(1)])
        .split(frame.size());

    // adding the shortcut
    frame.render_widget(
        Paragraph::new("   quit: q  |  up: k  | down: j | yank url: y")
            .style(SHORTCUT_STYLE)
            .left_aligned()
            .block(Block::new()),
        layout[1],
    );

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(2)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[0]);

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
