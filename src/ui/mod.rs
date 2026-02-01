pub mod component;
pub mod components;
pub mod highlight;
pub mod style;
pub mod testing;
pub mod theme;
pub mod utils;

// Legacy exports (to maintain compatibility during transition)
pub use style::Theme as LegacyTheme;
pub use utils::option_vec_to_option_slice;

// New component-based architecture exports
pub use component::{Component, ComponentLayout, LayoutComponent, TestableComponent};
pub use components::{
    ArticleListComponent, ConfigPopupComponent, FooterComponent, HelpPopupComponent,
    PinnedAuthorsComponent, PreviewComponent, SearchBarComponent,
};
pub use testing::GoldenTester;
pub use theme::Theme;

use crate::app::{App, Context};
use crate::arxiv::ArxivEntry;
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

    // 1. Primary Vertical Split: Left Column (Filter + Articles) | Right Column (Preview)
    let primary_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left column
            Constraint::Percentage(50), // Right column (full height preview)
        ])
        .split(layout[1]);

    // 2. Determine VIP feed constraint for left column sub-split
    // 1. Get the data you need (specifically just the authors and entries)
    let pinned = &app.config.pinned.authors;
    let entries = &app.query_result.articles;

    // 2. Filter them. This only borrows specific fields, not the whole 'app'
    let vip_articles: Vec<&ArxivEntry> = entries
        .iter()
        .filter(|e| {
            e.authors.iter().any(|author_name| {
                pinned
                    .iter()
                    .any(|p| author_name.to_lowercase().contains(&p.to_lowercase()))
            })
        })
        .collect();

    // 3. Now you can safely borrow the state mutably
    let vip_state = components::vip_feed::PinnedAuthorsState {
        vip_articles: &vip_articles,
        list_state: &mut app.vip_feed_state, // Rust is happy now!
        visible: !vip_articles.is_empty(),
        expanded: app.current_context == Context::Pinned,
    };
    let vip_component = PinnedAuthorsComponent::new();

    let vip_constraint_length = vip_component.get_constraint_length(&vip_state);

    // 3. Left Column Sub-split: VIP Feed (top) + Articles (bottom)
    let left_column_layout = if vip_constraint_length > 0 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(vip_constraint_length), // VIP feed
                Constraint::Min(0),                        // Articles list
            ])
            .split(primary_layout[0])
    } else {
        std::rc::Rc::from([primary_layout[0]]) // No VIP feed, articles use full left column
    };

    // 2. Component setup & Focus Management
    let mut search_bar = SearchBarComponent::new();
    let mut article_list = ArticleListComponent::new();
    let mut preview_component = PreviewComponent::new();
    let mut vip_feed_component = PinnedAuthorsComponent::new();

    match app.current_context {
        Context::Search => {
            search_bar.on_focus();
        }
        Context::ArticleList => {
            article_list.on_focus();
        }
        Context::Preview => {
            preview_component.on_focus();
        }
        Context::Pinned => {
            vip_feed_component.on_focus();
        }
        _ => {}
    }

    // 3. Rendering - Using the Component Trait

    // --- Render Search Bar ---
    let mut search_state = crate::ui::components::search_bar::SearchBarState {
        query: &app.search_state.query,
        visible: true,
    };
    search_bar.render(frame, layout[0], &mut search_state, &app.theme);

    // --- Render VIP Feed (if visible) ---
    if vip_constraint_length > 0 {
        let mut vip_state = components::vip_feed::PinnedAuthorsState {
            vip_articles: &vip_articles,
            list_state: &mut app.vip_feed_state,
            visible: true,
            expanded: app.current_context == Context::Pinned,
        };
        vip_feed_component.render(frame, left_column_layout[0], &mut vip_state, &app.theme);
    }

    // --- Render Article List ---
    // We save the selected index first to avoid multiple mutable borrows
    {
        let mut article_state = crate::ui::components::article_list::ArticleListState {
            query_result: app.query_result,
            list_state: &mut app.article_list_state,
            search_state: &app.search_state,
            search_engine: &mut app.search_engine,
            highlight_authors: Some(app.pinned_config.authors.as_slice()),
            scrollbar_state: ratatui::widgets::ScrollbarState::default(),
        };
        article_list.render(
            frame,
            if vip_constraint_length > 0 {
                left_column_layout[1]
            } else {
                left_column_layout[0]
            },
            &mut article_state,
            &app.theme,
        );
    }

    // --- Render Article Preview (Right Side) ---
    let preview_article = app.get_preview_article();
    let mut preview_state =
        components::preview::PreviewState::new(preview_article, app.pinned_config);
    preview_component.render(frame, primary_layout[1], &mut preview_state, &app.theme);

    // --- Render Footer ---
    let footer_component = FooterComponent::new();
    let mut footer_state = components::footer::FooterState {
        current_context: app.current_context,
        visible: true,
    };
    footer_component.render(frame, layout[2], &mut footer_state, &app.theme);

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
