use crate::arxiv::ArxivEntry;
use crate::ui::{component::Component, theme::Theme};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

#[derive(Debug, Clone, Default)]
pub struct PinnedAuthorsComponent {
    focused: bool,
    shortcut: Option<usize>,
}

pub struct PinnedAuthorsState<'a> {
    pub vip_articles: &'a [&'a ArxivEntry],
    pub list_state: &'a mut ListState,
    pub visible: bool,
    pub expanded: bool,
}

impl PinnedAuthorsComponent {
    pub fn new() -> Self {
        Self {
            focused: false,
            shortcut: Some(1),
        }
    }

    /// Create list items for VIP articles
    fn create_vip_article_items<'a>(
        &self,
        articles: &'a [&'a ArxivEntry],
        expanded: bool,
        theme: &Theme,
    ) -> Vec<ListItem<'a>> {
        articles
            .iter()
            .map(|article| {
                if expanded {
                    // Full article display when expanded (like main article list)
                    let title_line =
                        Line::from(Span::styled(article.title.clone(), theme.list.item));
                    let authors_line = Line::from(Span::styled(
                        article.get_all_authors().to_string(),
                        theme.list.date,
                    ));
                    let date_line = Line::from(Span::styled(
                        format!("Published: {}", article.published),
                        theme.list.date,
                    ));
                    ListItem::new(vec![title_line, authors_line, date_line])
                } else {
                    // Compact display when collapsed (just title)
                    let title = if article.title.len() > 50 {
                        format!("{}...", &article.title[..47])
                    } else {
                        article.title.clone()
                    };
                    ListItem::new(Line::from(Span::styled(title, theme.list.item)))
                }
            })
            .collect()
    }

    /// Determine if component should be visible based on state
    fn should_be_visible(&self, state: &PinnedAuthorsState) -> bool {
        state.visible && !state.vip_articles.is_empty()
    }

    /// Get the appropriate constraint length based on state (now for height)
    pub fn get_constraint_length(&self, state: &PinnedAuthorsState) -> u16 {
        // If we want it to stay visible even when empty, remove the !should_be_visible check
        if state.vip_articles.is_empty() && !self.focused {
            return 3; // Minimal strip height showing "VIP (0)"
        }

        if state.expanded {
            10
        } else {
            4
        }
    }
}

impl<'a> Component<'a> for PinnedAuthorsComponent {
    type State = PinnedAuthorsState<'a>;

    fn render(&self, frame: &mut Frame, area: Rect, state: &mut Self::State, theme: &Theme) {
        if !self.should_be_visible(state) {
            return;
        }

        let border_style = theme.get_border_style(self.focused, true);

        let label = if state.expanded {
            format!(" VIP Feed ({}) ", state.vip_articles.len())
        } else {
            format!(" VIP ({}) ", state.vip_articles.len())
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(theme.format_title(&label, self.shortcut, self.focused));

        let items = self.create_vip_article_items(state.vip_articles, state.expanded, theme);

        let highlight_style = if self.focused {
            theme.list.selected_focused
        } else {
            theme.list.selected_unfocused
        };

        let list = List::new(items)
            .block(block)
            .style(theme.list.item)
            .highlight_style(highlight_style)
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, state.list_state);
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
        (3, 3) // Minimum size for collapsed state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_pinned_authors_component_creation() {
        let component = PinnedAuthorsComponent::new();
        assert!(!component.focused);
    }

    #[test]
    fn test_constraint_length_calculation() {
        use crate::arxiv::ArxivEntry;

        let component = PinnedAuthorsComponent::new();
        let mut list_state = ListState::default();

        // Create mock VIP articles
        let article1 = ArxivEntry::new(
            "VIP Paper 1".to_string(),
            vec!["John Doe".to_string()],
            "Summary".to_string(),
            "id1".to_string(),
            "2024-01-01".to_string(),
            "2024-01-01".to_string(),
        );
        let vip_articles = vec![&article1];

        // Test expanded state
        let state = PinnedAuthorsState {
            vip_articles: &vip_articles,
            list_state: &mut list_state,
            visible: true,
            expanded: true,
        };
        assert_eq!(component.get_constraint_length(&state), 10);

        // Test collapsed state
        let state = PinnedAuthorsState {
            vip_articles: &vip_articles,
            list_state: &mut list_state,
            visible: true,
            expanded: false,
        };
        assert_eq!(component.get_constraint_length(&state), 4);

        // Test hidden state (empty articles)
        let empty_articles: Vec<&ArxivEntry> = vec![];
        let state = PinnedAuthorsState {
            vip_articles: &empty_articles,
            list_state: &mut list_state,
            visible: true,
            expanded: true,
        };
        assert_eq!(component.get_constraint_length(&state), 3);
    }

    #[test]
    fn test_vip_article_items_creation() {
        use crate::arxiv::ArxivEntry;

        let component = PinnedAuthorsComponent::new();
        let theme = Theme::default();

        let article1 = ArxivEntry::new(
            "VIP Paper 1".to_string(),
            vec!["John Doe".to_string()],
            "Summary".to_string(),
            "id1".to_string(),
            "2024-01-01".to_string(),
            "2024-01-01".to_string(),
        );
        let article2 = ArxivEntry::new(
            "VIP Paper 2".to_string(),
            vec!["Jane Smith".to_string()],
            "Summary".to_string(),
            "id2".to_string(),
            "2024-01-02".to_string(),
            "2024-01-02".to_string(),
        );
        let vip_articles = vec![&article1, &article2];

        // Test expanded items (full article display)
        let expanded_items = component.create_vip_article_items(&vip_articles, true, &theme);
        assert_eq!(expanded_items.len(), 2);

        // Test collapsed items (compact display)
        let collapsed_items = component.create_vip_article_items(&vip_articles, false, &theme);
        assert_eq!(collapsed_items.len(), 2);
    }

    #[test]
    fn test_render_with_empty_vip_articles() {
        let component = PinnedAuthorsComponent::new();
        let theme = Theme::default();
        let empty_articles: Vec<&ArxivEntry> = vec![];
        let mut list_state = ListState::default();
        let mut state = PinnedAuthorsState {
            vip_articles: &empty_articles,
            list_state: &mut list_state,
            visible: true,
            expanded: true,
        };

        let area = Rect::new(0, 0, 30, 10);
        let backend = TestBackend::new(30, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|frame| {
            component.render(frame, area, &mut state, &theme);
        });

        assert!(result.is_ok());
        // Component should not render anything when VIP articles list is empty
    }
}
