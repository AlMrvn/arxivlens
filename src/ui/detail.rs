use crate::arxiv::ArxivEntry;
use crate::config::HighlightConfig;
use crate::search_highlight::highlight_patterns;
use crate::ui::Theme;

use super::option_vec_to_option_slice;
use itertools::izip;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
    Frame,
};

pub struct ArticleDetails<'a> {
    title: Line<'a>,
    authors: Line<'a>,
    summary: Line<'a>,
}

impl<'a> ArticleDetails<'a> {
    pub fn new(entry: &'a ArxivEntry, highlight_config: &HighlightConfig, theme: &Theme) -> Self {
        let author_patterns = option_vec_to_option_slice(&highlight_config.authors);
        let keyword_patterns = option_vec_to_option_slice(&highlight_config.keywords);
        Self {
            title: highlight_patterns(&entry.title, keyword_patterns.as_deref(), theme),
            authors: highlight_patterns(
                entry.get_all_authors(),
                author_patterns.as_deref(),
                theme,
            ),
            summary: highlight_patterns(&entry.summary, keyword_patterns.as_deref(), theme),
        }
    }

    pub fn render(self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let sub_layout = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(2)
            .constraints([
                Constraint::Length(4), // Title
                Constraint::Length(6), // Authors
                Constraint::Min(10),   // Abstract/summary
            ])
            .split(area);

        let titles_sec = vec![" Title ", " Author ", " Abstract "];
        let areas = vec![sub_layout[0], sub_layout[1], sub_layout[2]];
        let items = vec![&self.title, &self.authors, &self.summary];

        for (title, entry, area) in izip!(titles_sec, items, areas) {
            frame.render_widget(
                Paragraph::new(entry.clone())
                    .block(
                        Block::new()
                            .borders(Borders::TOP)
                            .title(title)
                            .title_style(theme.title)
                            .title_alignment(Alignment::Left)
                            .border_type(BorderType::Plain)
                            .padding(Padding::horizontal(2)),
                    )
                    .style(theme.main)
                    .left_aligned()
                    .wrap(Wrap { trim: true }),
                area,
            )
        }
    }
}
