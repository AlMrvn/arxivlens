pub mod component;
pub mod components;
pub mod detail;
pub mod footer;
pub mod highlight;
pub mod search;
pub mod style;
pub mod testing;
pub mod theme;
pub mod utils;

// Legacy exports (to maintain compatibility during transition)
pub use detail::ArticleDetails;
pub use footer::render_footer;
pub use search::render_search_bar;
pub use style::Theme as LegacyTheme;
pub use utils::option_vec_to_option_slice;

// New component-based architecture exports
pub use component::{Component, ComponentLayout, LayoutComponent, TestableComponent};
pub use components::{
    ArticleListComponent, ConfigPopupComponent, HelpPopupComponent, SearchBarComponent,
};
pub use testing::GoldenTester;
pub use theme::Theme;

use crate::app::{App, Context};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

/// Render the app:
pub fn render(app: &mut App, frame: &mut Frame) {
    // 1. Layout logic
    let area = frame.size();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Search bar
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Footer
        ])
        .split(area);

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);

    // 2. Component setup & Focus Management
    let mut search_bar = SearchBarComponent::new();
    let mut article_list = ArticleListComponent::new();

    if matches!(app.current_context, Context::Search) {
        search_bar.on_focus();
        article_list.on_blur();
    } else {
        search_bar.on_blur();
        article_list.on_focus();
    }

    // 3. Rendering - Using the Component Trait

    // --- Render Search Bar ---
    let mut search_state = crate::ui::components::search_bar::SearchBarState {
        query: &app.search_state.query,
        visible: true,
    };
    search_bar.render(frame, layout[0], &mut search_state, &app.theme);

    // --- Render Article List ---
    // We save the selected index first to avoid multiple mutable borrows
    let selected_index = app.article_list_state.selected();
    let visible_count = app.get_visible_count();

    {
        let mut article_state = crate::ui::components::article_list::ArticleListState {
            query_result: app.query_result,
            list_state: &mut app.article_list_state,
            search_state: &app.search_state,
            search_engine: &mut app.search_engine,
            highlight_authors: app.highlight_config.authors.as_deref(),
            scrollbar_state: ratatui::widgets::ScrollbarState::default(),
        };
        article_list.render(frame, main_layout[0], &mut article_state, &app.theme);
    }

    // --- Render Article Details (Right Side) ---
    // We use the app helper to find the currently selected article
    if let Some(entry) = app.get_selected_article_by_index(selected_index, visible_count) {
        let details = ArticleDetails::new(entry, app.highlight_config, &app.theme);
        details.render(frame, main_layout[1], &app.theme);
    } else {
        ArticleDetails::no_results(&app.theme).render(frame, main_layout[1], &app.theme);
    }

    // --- Render Footer ---
    render_footer(frame, layout[2], app);

    // --- Render Overlays (Popups) ---
    render_overlays(app, frame);
}

fn render_overlays(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    match app.current_context {
        Context::Config => {
            let mut config_comp = ConfigPopupComponent::new();
            config_comp.on_focus();
            let mut config_state = crate::ui::components::config_popup::ConfigPopupState {
                config: &app.config,
                visible: true,
            };
            config_comp.render(frame, area, &mut config_state, &app.theme);
        }
        Context::Help => {
            let mut help_comp = HelpPopupComponent::new();
            help_comp.on_focus();
            let mut help_state =
                crate::ui::components::help_popup::HelpPopupState { visible: true };
            help_comp.render(frame, area, &mut help_state, &app.theme);
        }
        _ => {}
    }
}
