use crate::config::Config;
use crate::ui::component::{Component, TestableComponent};
use crate::ui::theme::Theme;
use ratatui::{layout::Rect, widgets::Block, Frame};

#[derive(Debug, Clone, Default)]
pub struct ConfigPopupComponent {
    focused: bool,
}

pub struct ConfigPopupState<'a> {
    pub config: &'a Config,
    pub visible: bool,
}

impl ConfigPopupComponent {
    pub fn new() -> Self {
        Self { focused: false }
    }
}

impl Component for ConfigPopupComponent {
    type State = ConfigPopupState<'static>;

    fn render(&self, frame: &mut Frame, area: Rect, state: &mut Self::State, theme: &Theme) {
        if !state.visible {
            return;
        }

        let popup_area = theme.centered_popup_area(60, 50, area);

        let block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_set(theme.border.set)
            .border_style(theme.get_border_style(self.focused, true))
            .title("Configuration")
            .title_style(theme.popup.title)
            .style(theme.popup.background);

        frame.render_widget(block, popup_area);

        // TODO: Implement actual config rendering
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

impl TestableComponent for ConfigPopupComponent {
    fn create_test_instance() -> Self {
        Self::new()
    }

    fn get_test_state() -> Self::State {
        use crate::config::Config;

        // Create a mock config and leak it for 'static lifetime
        let config = Box::leak(Box::new(Config::default()));

        ConfigPopupState {
            config,
            visible: true,
        }
    }

    fn test_name() -> &'static str {
        "config_popup"
    }
}
