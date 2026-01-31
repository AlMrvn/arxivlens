use crate::ui::component::{Component, TestableComponent};
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    widgets::{Block, Paragraph},
    Frame,
};

#[derive(Debug, Clone, Default)]
pub struct SearchBarComponent {
    focused: bool,
}

pub struct SearchBarState<'a> {
    pub query: &'a str,
    pub visible: bool,
}

impl SearchBarComponent {
    pub fn new() -> Self {
        Self { focused: false }
    }
}

impl Component for SearchBarComponent {
    type State = SearchBarState<'static>;

    fn render(&self, frame: &mut Frame, area: Rect, state: &mut Self::State, theme: &Theme) {
        if !state.visible {
            return;
        }

        let block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_set(theme.border.set)
            .border_style(theme.get_border_style(self.focused, true))
            .title("Search")
            .title_style(theme.title);

        let paragraph = Paragraph::new(state.query)
            .block(block)
            .style(theme.search.input);

        frame.render_widget(paragraph, area);
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

impl TestableComponent for SearchBarComponent {
    fn create_test_instance() -> Self {
        Self::new()
    }

    fn get_test_state() -> Self::State {
        SearchBarState {
            query: "test query",
            visible: true,
        }
    }

    fn test_name() -> &'static str {
        "search_bar"
    }
}
