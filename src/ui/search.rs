use crate::app::App;
use crate::ui::Theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the search bar with current query and match count
pub fn render_search_bar(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let query = &app.search_state.query;
    let count = app.search_state.filtered_count();
    let total = app.query_result.articles.len();

    let search_text = if query.is_empty() {
        Line::from(vec![
            Span::raw("/"),
            Span::styled(
                " Type to search... (Enter/Esc to exit)",
                Style::default().fg(Color::DarkGray),
            ),
        ])
    } else {
        Line::from(vec![Span::raw("/"), Span::styled(query, theme.main)])
    };

    // Create the match counter as a separate paragraph for right alignment
    let counter_text = if !query.is_empty() {
        format!("[{}/{}]", count, total)
    } else {
        String::new()
    };

    let search_paragraph = Paragraph::new(search_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
                .style(theme.main),
        )
        .style(theme.main);

    let counter_paragraph = Paragraph::new(counter_text)
        .style(theme.shortcut)
        .alignment(Alignment::Right);

    frame.render_widget(search_paragraph, area);

    // Render the counter in the same area but right-aligned
    if !query.is_empty() {
        // Create a smaller area for the counter, accounting for borders
        let counter_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: 1,
        };
        frame.render_widget(counter_paragraph, counter_area);
    }

    // Set cursor position in the search input
    frame.set_cursor(area.x + query.len() as u16 + 2, area.y + 1);
}
