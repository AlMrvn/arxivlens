use super::super::component::{Component, TestableComponent};
use super::super::utils::highlight_patterns;
use super::super::Theme;
use crate::arxiv::ArxivEntry;
use crate::config::PinnedConfig;

use itertools::izip;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
    Frame,
};

/// State for the preview component
pub struct PreviewState<'a> {
    pub article: Option<&'a ArxivEntry>,
    pub pinned_config: &'a PinnedConfig,
}

impl<'a> PreviewState<'a> {
    pub fn new(article: Option<&'a ArxivEntry>, pinned_config: &'a PinnedConfig) -> Self {
        Self {
            article,
            pinned_config,
        }
    }

    pub fn with_article(&mut self, article: Option<&'a ArxivEntry>) {
        self.article = article;
    }
}

/// Preview component for displaying article details
#[derive(Debug)]
pub struct PreviewComponent {
    focused: bool,
    shortcut: Option<usize>,
}

/// Content data for rendering sections
struct SectionContent<'a> {
    title: &'a Line<'a>,
    authors: &'a Line<'a>,
    summary: &'a Line<'a>,
    updated: &'a Line<'a>,
}

impl PreviewComponent {
    pub fn new() -> Self {
        Self {
            focused: false,
            shortcut: Some(3),
        }
    }

    /// Create the content lines for an article
    fn create_article_lines<'a>(
        &self,
        entry: &'a ArxivEntry,
        pinned_config: &PinnedConfig,
        theme: &Theme,
    ) -> (Line<'a>, Line<'a>, Line<'a>, Line<'a>) {
        let author_patterns: Vec<&str> = pinned_config.authors.iter().map(|s| s.as_str()).collect();
        let title = highlight_patterns(&entry.title, None, theme);
        let authors = highlight_patterns(entry.get_all_authors(), Some(&author_patterns), theme);
        let summary = highlight_patterns(&entry.summary, None, theme);
        let updated = Line::from(entry.updated.clone());

        (title, authors, summary, updated)
    }

    /// Create "No results found" content
    fn create_no_results_lines(
        &self,
        theme: &Theme,
    ) -> (Line<'static>, Line<'static>, Line<'static>, Line<'static>) {
        let title = Line::raw("No results found").style(theme.main);
        let authors = Line::raw("").style(theme.main);
        let summary = Line::raw("No articles match your current search criteria.\n\nYou can:\n• Try different keywords\n• Check for typos\n• Use broader search terms\n• Clear the search to see all articles").style(theme.main);
        let updated = Line::raw("").style(theme.main);

        (title, authors, summary, updated)
    }

    /// Render the content sections
    fn render_sections(
        &self,
        frame: &mut Frame,
        area: Rect,
        content: SectionContent,
        theme: &Theme,
    ) {
        let sub_layout = Layout::default()
            .direction(Direction::Vertical)
            // Use the centralized theme value here!
            .margin(theme.layout.padding)
            .constraints([
                Constraint::Length(4),
                Constraint::Length(6),
                Constraint::Min(10),
                Constraint::Length(2),
            ])
            .split(area);

        let titles_sec = vec![" Title ", " Author ", " Abstract ", "Updated"];
        let areas = vec![sub_layout[0], sub_layout[1], sub_layout[2], sub_layout[3]];
        let items = vec![
            content.title,
            content.authors,
            content.summary,
            content.updated,
        ];

        for (section_title, item_content, area) in izip!(titles_sec, items, areas) {
            frame.render_widget(
                Paragraph::new(item_content.clone())
                    .block(
                        Block::new()
                            .borders(Borders::TOP)
                            .title(section_title)
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

impl<'a> Component<'a> for PreviewComponent {
    type State = PreviewState<'a>;

    fn render(&self, frame: &mut Frame, area: Rect, state: &mut Self::State, theme: &Theme) {
        // Determine the border color based on focus
        let border_style = theme.get_border_style(self.focused, true);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(theme.format_title(" Article Preview ", self.shortcut, self.focused));

        // Get the area inside the borders
        let inner_area = block.inner(area);

        // Render the container
        frame.render_widget(block, area);

        // Render content inside inner_area
        let (title, authors, summary, updated) = match state.article {
            Some(article) => self.create_article_lines(article, state.pinned_config, theme),
            None => self.create_no_results_lines(theme),
        };

        let content = SectionContent {
            title: &title,
            authors: &authors,
            summary: &summary,
            updated: &updated,
        };

        self.render_sections(frame, inner_area, content, theme);
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_focus(&mut self) {
        self.focused = true;
    }

    fn on_blur(&mut self) {
        self.focused = false;
    }

    fn min_size(&self) -> (u16, u16) {
        (40, 20) // Minimum size for readable article preview
    }
}

impl TestableComponent<'_> for PreviewComponent {
    fn create_test_instance() -> Self {
        Self::new()
    }

    fn get_test_state() -> Self::State {
        use crate::config::PinnedConfig;
        use std::sync::OnceLock;

        static TEST_PINNED_CONFIG: OnceLock<PinnedConfig> = OnceLock::new();
        let pinned_config = TEST_PINNED_CONFIG.get_or_init(|| PinnedConfig {
            authors: vec!["Test Author".to_string()],
            categories: vec!["quant-ph".to_string()],
        });

        PreviewState {
            article: None, // Will be set in actual tests
            pinned_config,
        }
    }

    fn test_name() -> &'static str {
        "PreviewComponent"
    }
}

impl Default for PreviewComponent {
    fn default() -> Self {
        Self::new()
    }
}
