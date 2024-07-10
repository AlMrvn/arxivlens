use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{
        self, Block, BorderType, HighlightSpacing, List, ListDirection, ListItem, Paragraph,
    },
    Frame,
};

use crate::app::App;
use crate::arxiv_entry;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples

    // First we create a Layout
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.size());

    let sub_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(10)])
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
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom)
        .highlight_spacing(HighlightSpacing::Always);

    // The selection windows
    frame.render_stateful_widget(list, layout[0], &mut app.arxiv_entries.state);

    // The abstract of the manuscript
    let authors = if let Some(i) = app.arxiv_entries.state.selected() {
        app.arxiv_entries.items[i].authors.join(", ")
    } else {
        "Nothin selected...".to_string()
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
            .centered(),
        sub_layout[0],
    );

    // The abstract of the manuscript
    let summary = if let Some(i) = app.arxiv_entries.state.selected() {
        app.arxiv_entries.items[i].summary.clone()
    } else {
        "Nothin selected...".to_string()
    };

    frame.render_widget(
        Paragraph::new(format!("{}", summary))
            .block(
                Block::bordered()
                    .title("Abstract")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::Cyan).bg(Color::Black))
            .left_aligned(),
        sub_layout[1],
    )
}
