use crate::ui::theme::Theme;
use ratatui::{layout::Rect, Frame};

/// Core trait for all UI components in the application
pub trait Component<'a> {
    /// The state type this component operates on
    type State;

    /// Render the component to the given frame and area
    fn render(&self, frame: &mut Frame, area: Rect, state: &mut Self::State, theme: &Theme);

    /// Handle component-specific input events
    /// Returns true if the event was handled, false otherwise
    fn handle_event(
        &mut self,
        _event: &ratatui::crossterm::event::Event,
        _state: &mut Self::State,
    ) -> bool {
        false // Default implementation - no event handling
    }

    /// Get the minimum size required for this component
    fn min_size(&self) -> (u16, u16) {
        (1, 1) // Default minimum size
    }

    /// Whether this component should receive focus
    fn can_focus(&self) -> bool {
        false
    }

    /// Called when component gains focus
    fn on_focus(&mut self) {}

    /// Called when component loses focus
    fn on_blur(&mut self) {}
}

/// Trait for components that can be tested with golden files
pub trait TestableComponent<'a>: Component<'a> {
    /// Create a test instance of this component with mock data
    fn create_test_instance() -> Self;

    /// Get test state for golden file testing
    fn get_test_state() -> Self::State;

    /// Get a descriptive name for test snapshots
    fn test_name() -> &'static str;
}

/// Helper trait for components that manage their own layout
pub trait LayoutComponent<'a>: Component<'a> {
    /// Calculate the internal layout for this component
    fn calculate_layout(&self, area: Rect) -> ComponentLayout;
}

/// Standard layout structure for components
#[derive(Debug, Clone)]
pub struct ComponentLayout {
    pub border: Option<Rect>,
    pub content: Rect,
    pub title: Option<Rect>,
    pub footer: Option<Rect>,
}

impl ComponentLayout {
    pub fn new(area: Rect) -> Self {
        Self {
            border: None,
            content: area,
            title: None,
            footer: None,
        }
    }

    pub fn with_border(mut self, border_area: Rect, content_area: Rect) -> Self {
        self.border = Some(border_area);
        self.content = content_area;
        self
    }

    pub fn with_title(mut self, title_area: Rect) -> Self {
        self.title = Some(title_area);
        self
    }

    pub fn with_footer(mut self, footer_area: Rect) -> Self {
        self.footer = Some(footer_area);
        self
    }
}
