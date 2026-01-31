use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::app::search::SearchState;
use crate::arxiv::ArxivQueryResult;
use crate::ui::component::{Component, ComponentLayout, LayoutComponent, TestableComponent};
use crate::ui::theme::Theme;
use crate::ui::utils::highlight_patterns;

#[derive(Debug, Clone, Default)]
pub struct ArticleListComponent {
    focused: bool,
}

pub struct ArticleListState<'a> {
    pub query_result: &'a ArxivQueryResult,
    pub list_state: &'a mut ListState,
    pub search_state: &'a SearchState,
    pub highlight_authors: Option<&'a [String]>,
    pub scrollbar_state: ScrollbarState,
}

impl ArticleListComponent {
    pub fn new() -> Self {
        Self { focused: false }
    }
    fn create_list_items<'a>(
        &self,
        state: &ArticleListState<'a>,
        theme: &Theme,
    ) -> Vec<ListItem<'a>> {
        let articles = if state.search_state.is_active() {
            // Get filtered articles based on search
            state
                .search_state
                .filtered_indices
                .iter()
                .filter_map(|&idx| state.query_result.articles.get(idx))
                .collect::<Vec<_>>()
        } else {
            state.query_result.articles.iter().collect()
        };

        // Convert highlight_authors to owned strings for pattern matching
        let highlight_patterns_vec: Option<Vec<String>> = state
            .highlight_authors
            .map(|authors| authors.iter().map(|s| s.to_string()).collect());

        articles
            .into_iter()
            .map(|article| {
                let title_line = if state.search_state.is_active() {
                    highlight_patterns(&article.title, None, theme)
                } else {
                    Line::from(Span::styled(article.title.clone(), theme.list.item))
                };

                let authors_patterns: Option<Vec<&str>> = highlight_patterns_vec
                    .as_ref()
                    .map(|v| v.iter().map(|s| s.as_str()).collect());

                // If highlight_patterns returns Line<'a>, 'a cannot be local 'authors_text'
                // We must pass a reference that outlives this closure iteration.
                // If 'article' is already a reference from 'state', use it directly:
                let authors_line = highlight_patterns(
                    article.get_all_authors(),
                    authors_patterns.as_deref(),
                    theme,
                );

                let date_line = Line::from(Span::styled(
                    format!("Published: {}", article.published),
                    theme.list.date,
                ));

                ListItem::new(vec![title_line, authors_line, date_line])
            })
            .collect()
    }
}

impl Component for ArticleListComponent {
    type State = ArticleListState<'static>;

    fn render(&self, frame: &mut Frame, area: Rect, state: &mut Self::State, theme: &Theme) {
        let layout = self.calculate_layout(area);

        // Render border if present
        if let Some(border_area) = layout.border {
            let border_style = theme.get_border_style(self.focused, true);
            let block = Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_set(theme.border.set)
                .border_style(border_style)
                .title("Articles")
                .title_style(theme.title);
            frame.render_widget(block, border_area);
        }

        // Create and render the list
        let items = self.create_list_items(state, theme);
        let list = List::new(items)
            .highlight_style(theme.list.selected)
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, layout.content, state.list_state);

        // Update and render scrollbar
        let visible_count = if state.search_state.is_active() {
            state.search_state.filtered_count()
        } else {
            state.query_result.articles.len()
        };

        state.scrollbar_state = state
            .scrollbar_state
            .content_length(visible_count)
            .position(state.list_state.selected().unwrap_or(0));

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .style(theme.list.scrollbar);

        frame.render_stateful_widget(scrollbar, area, &mut state.scrollbar_state);
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

impl LayoutComponent for ArticleListComponent {
    fn calculate_layout(&self, area: Rect) -> ComponentLayout {
        let border_area = area;
        let content_area = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(100)])
            .split(area)[0];

        ComponentLayout::new(area).with_border(border_area, content_area)
    }
}

impl TestableComponent for ArticleListComponent {
    fn create_test_instance() -> Self {
        Self::new()
    }

    fn get_test_state() -> Self::State {
        use crate::app::search::SearchState;
        use crate::arxiv::{ArxivEntry, ArxivQueryResult};
        use ratatui::widgets::ListState;

        // Create mock articles
        let articles = vec![
            ArxivEntry::new(
                "Quantum Computing Advances".to_string(),
                vec!["Alice Smith".to_string(), "Bob Johnson".to_string()],
                "Recent advances in quantum computing algorithms and their applications."
                    .to_string(),
                "2024.01001".to_string(),
                "2024-01-15".to_string(),
                "2024-01-15".to_string(),
            ),
            ArxivEntry::new(
                "Machine Learning in Physics".to_string(),
                vec!["Carol Davis".to_string(), "David Wilson".to_string()],
                "Applications of machine learning techniques in theoretical physics.".to_string(),
                "2024.01002".to_string(),
                "2024-01-16".to_string(),
                "2024-01-16".to_string(),
            ),
            ArxivEntry::new(
                "Neural Networks and Optimization".to_string(),
                vec!["Eve Brown".to_string()],
                "Novel optimization techniques for deep neural networks.".to_string(),
                "2024.01003".to_string(),
                "2024-01-17".to_string(),
                "2024-01-17".to_string(),
            ),
        ];

        // Create mock query result and leak it for 'static lifetime
        let query_result = Box::leak(Box::new(ArxivQueryResult {
            updated: "2024-01-17T12:00:00Z".to_string(),
            articles,
        }));

        // Create mock list state and leak it
        let list_state = Box::leak(Box::new(ListState::default()));

        // Create mock search state and leak it
        let search_state = Box::leak(Box::new(SearchState::new()));

        // Create mock highlight authors and leak it
        let highlight_authors = Box::leak(Box::new(vec![
            "Alice Smith".to_string(),
            "Bob Johnson".to_string(),
        ]));

        ArticleListState {
            query_result,
            list_state,
            search_state,
            highlight_authors: Some(highlight_authors),
            scrollbar_state: ScrollbarState::default(),
        }
    }

    fn test_name() -> &'static str {
        "article_list"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_list_component_creation() {
        let component = ArticleListComponent::new();
        assert!(component.can_focus());
        assert!(!component.focused);
    }

    #[test]
    fn test_focus_management() {
        let mut component = ArticleListComponent::new();

        component.on_focus();
        assert!(component.focused);

        component.on_blur();
        assert!(!component.focused);
    }

    #[test]
    fn test_layout_calculation() {
        let component = ArticleListComponent::new();
        let area = Rect::new(0, 0, 80, 24);
        let layout = component.calculate_layout(area);

        assert!(layout.border.is_some());
        assert_eq!(layout.content.width, area.width - 2); // Account for borders
        assert_eq!(layout.content.height, area.height - 2);
    }
}
