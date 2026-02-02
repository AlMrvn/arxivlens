use crate::arxiv::ArxivEntry;
use crate::search::engine::SearchEngine;
use crate::ui::component::{Component, TestableComponent};
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};

#[derive(Debug, Clone, Default)]
pub struct ArticleFeed {
    pub focused: bool,
    pub shortcut: Option<usize>,
    pub title_label: String,
}

pub struct ArticleFeedState<'a> {
    pub articles: Vec<&'a ArxivEntry>,
    pub list_state: &'a mut ListState,
    pub scrollbar_state: &'a mut ScrollbarState,
    /// Current search query for fuzzy highlighting
    pub search_query: Option<&'a str>,
    pub search_engine: Option<&'a mut SearchEngine>,
    /// Authors to highlight across all instances
    pub watched_authors: Option<&'a [String]>,
}

impl ArticleFeed {
    pub fn new(title_label: impl Into<String>, shortcut: Option<usize>) -> Self {
        Self {
            focused: false,
            shortcut,
            title_label: title_label.into(),
        }
    }

    fn create_items<'a>(
        &self,
        state: &mut ArticleFeedState<'a>,
        theme: &Theme,
    ) -> Vec<ListItem<'a>> {
        state
            .articles
            .iter()
            .map(|article| {
                // 1. Title Highlighting
                let title_line = if let (Some(query), Some(engine)) =
                    (state.search_query, &mut state.search_engine)
                {
                    let indices = engine.get_highlight_indices(query, &article.title);
                    crate::ui::highlight::Highlighter::fuzzy_line(&article.title, &indices, theme)
                } else {
                    Line::from(Span::styled(article.title.clone(), theme.list.item))
                };

                // 2. Author Highlighting - Use the dedicated theme style
                let authors_text = article.get_all_authors();
                let author_patterns: Option<Vec<&str>> = state
                    .watched_authors
                    .map(|authors| authors.iter().map(|s| s.as_str()).collect());

                // We use the theme's author style (LIGHT_GRAY) as the base
                let mut authors_line = crate::ui::utils::highlight_patterns(
                    authors_text,
                    author_patterns.as_deref(),
                    theme,
                    theme.list.authors,
                );

                // Force the base style of the author line to be the dim theme style
                // This ensures un-highlighted authors are subtle
                authors_line.style = theme.list.authors;

                ListItem::new(vec![title_line, authors_line])
            })
            .collect()
    }
}

impl<'a> Component<'a> for ArticleFeed {
    type State = ArticleFeedState<'a>;

    fn render(&self, frame: &mut Frame, area: Rect, state: &mut Self::State, theme: &Theme) {
        // Hide if empty and not focused (keeps the UI clean)
        if state.articles.is_empty() && !self.focused {
            return;
        }

        let items = self.create_items(state, theme);
        let border_style = theme.get_border_style(self.focused, true);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(border_style)
            .title(theme.format_title(
                &self.title_label,
                self.shortcut,
                self.focused,
                Some(state.articles.len()),
            ));

        let highlight_style = if self.focused {
            theme.list.selected_focused
        } else {
            theme.list.selected_unfocused
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(highlight_style)
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, state.list_state);

        // 1. Calculate how many items are actually visible in the area
        // We subtract 2 for the top and bottom borders of the block
        let height = area.height.saturating_sub(2);

        // 2. Update the scrollbar state using internal mutation
        let _ = state.scrollbar_state.content_length(state.articles.len());
        let _ = state
            .scrollbar_state
            .viewport_content_length(height as usize);
        let _ = state
            .scrollbar_state
            .position(state.list_state.selected().unwrap_or(0));

        // 3. Render it (passing the mutable reference)
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .symbols(ratatui::symbols::scrollbar::VERTICAL)
                .style(theme.list.scrollbar),
            area,
            state.scrollbar_state, // Use the reference directly
        );
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
}

impl TestableComponent<'_> for ArticleFeed {
    fn create_test_instance() -> Self {
        Self::new("Test Feed", Some(1))
    }

    fn get_test_state() -> Self::State {
        let list_state = Box::leak(Box::new(ratatui::widgets::ListState::default()));
        let scrollbar_state = Box::leak(Box::new(ratatui::widgets::ScrollbarState::default()));

        ArticleFeedState {
            articles: vec![],
            list_state,
            scrollbar_state,
            search_engine: None,
            search_query: None,
            watched_authors: None,
        }
    }

    fn test_name() -> &'static str {
        "ArticleFeed"
    }
}
