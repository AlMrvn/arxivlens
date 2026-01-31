use crate::ui::component::{Component, TestableComponent};
use crate::ui::theme::Theme;
use ratatui::{layout::Rect, widgets::Block, Frame};

#[derive(Debug, Clone, Default)]
pub struct HelpPopupComponent {
    focused: bool,
}

pub struct HelpPopupState {
    pub visible: bool,
}

impl HelpPopupComponent {
    pub fn new() -> Self {
        Self { focused: false }
    }
}

impl Component for HelpPopupComponent {
    type State = HelpPopupState;

    fn render(&self, frame: &mut Frame, area: Rect, state: &mut Self::State, theme: &Theme) {
        if !state.visible {
            return;
        }

        let popup_area = theme.centered_popup_area(70, 60, area);

        let block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_set(theme.border.set)
            .border_style(theme.get_border_style(self.focused, true))
            .title("Help")
            .title_style(theme.popup.title)
            .style(theme.popup.background);

        frame.render_widget(block, popup_area);

        // TODO: Implement actual help content rendering
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

impl TestableComponent for HelpPopupComponent {
    fn create_test_instance() -> Self {
        Self::new()
    }

    fn get_test_state() -> Self::State {
        HelpPopupState { visible: true }
    }

    fn test_name() -> &'static str {
        "help_popup"
    }
}
