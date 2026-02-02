pub mod component;
pub mod components;
pub mod highlight;
pub mod style;
pub mod testing;
pub mod theme;
pub mod utils;

// Legacy exports
pub use style::Theme as LegacyTheme;
pub use utils::option_vec_to_option_slice;

// New component-based architecture exports
pub use component::{Component, ComponentLayout, LayoutComponent, TestableComponent};
pub use components::{
    ArticleFeed, ConfigPopupComponent, FooterComponent, HelpPopupComponent, PreviewComponent,
    SearchBarComponent,
};
pub use testing::GoldenTester;
pub use theme::Theme;

use crate::app::{App, Context};
use crate::arxiv::ArxivEntry;
use crate::ui::components::article_feed::ArticleFeedState;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

/// Render the app:
pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.size();

    // 1. Root Vertical Layout
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Search bar
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // 2. Main Horizontal Split (Feeds | Preview)
    let primary_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);

    // 3. Data Preparation
    let pinned = &app.config.pinned.authors;

    // Filter VIP articles
    let vip_articles: Vec<&ArxivEntry> = app
        .query_result
        .articles
        .iter()
        .filter(|e| {
            e.authors.iter().any(|author_name| {
                pinned
                    .iter()
                    .any(|p| author_name.to_lowercase().contains(&p.to_lowercase()))
            })
        })
        .collect();

    // Filter Main feed articles
    let main_articles: Vec<&ArxivEntry> = if app.search_state.is_active() {
        app.search_state
            .filtered_indices
            .iter()
            .filter_map(|&idx| app.query_result.articles.get(idx))
            .collect()
    } else {
        app.query_result.articles.iter().collect()
    };

    // 4. Determine Layout Heights
    let is_pinned_focused = app.current_context == Context::Pinned;
    let vip_height = if vip_articles.is_empty() {
        0
    } else if is_pinned_focused {
        10 // Expanded height when focused
    } else {
        4 // Minimal strip height
    };

    let left_column_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(vip_height), Constraint::Min(0)])
        .split(primary_layout[0]);

    // 5. Component Initialization & Rendering

    // --- Search Bar ---
    let mut search_bar = SearchBarComponent::new();
    if app.current_context == Context::Search {
        search_bar.on_focus();
    }
    search_bar.render(
        frame,
        layout[0],
        &mut crate::ui::components::search_bar::SearchBarState {
            query: &app.search_state.query,
            visible: true,
        },
        &app.theme,
    );

    // --- VIP Feed ---
    if vip_height > 0 {
        let mut vip_feed = ArticleFeed::new(" VIP Feed ", Some(1));
        if is_pinned_focused {
            vip_feed.on_focus();
        }

        let mut vip_state = ArticleFeedState {
            articles: vip_articles,
            list_state: &mut app.vip_feed_state,
            scrollbar_state: &mut ratatui::widgets::ScrollbarState::default(),
            search_query: Some(&app.search_state.query),
            search_engine: Some(&mut app.search_engine),
            watched_authors: Some(&app.config.pinned.authors),
        };
        vip_feed.render(frame, left_column_layout[0], &mut vip_state, &app.theme);
    }

    // --- Main Article Feed ---
    let mut main_feed = ArticleFeed::new(" Articles ", Some(2));
    if app.current_context == Context::ArticleList {
        main_feed.on_focus();
    }

    let mut main_state = ArticleFeedState {
        articles: main_articles,
        list_state: &mut app.article_list_state,
        scrollbar_state: &mut app.article_scrollbar_state,
        search_query: Some(&app.search_state.query),
        search_engine: Some(&mut app.search_engine),
        watched_authors: Some(&app.config.pinned.authors),
    };
    main_feed.render(
        frame,
        if vip_height > 0 {
            left_column_layout[1]
        } else {
            left_column_layout[0]
        },
        &mut main_state,
        &app.theme,
    );

    // --- Preview ---
    let mut preview = PreviewComponent::new();
    if app.current_context == Context::Preview {
        preview.on_focus();
    }
    preview.render(
        frame,
        primary_layout[1],
        &mut crate::ui::components::preview::PreviewState::new(
            app.get_preview_article(),
            app.pinned_config,
        ),
        &app.theme,
    );

    // --- Footer ---
    FooterComponent::new().render(
        frame,
        layout[2],
        &mut crate::ui::components::footer::FooterState {
            current_context: app.current_context,
            visible: true,
        },
        &app.theme,
    );

    render_overlays(app, frame);
}

fn render_overlays(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    match app.current_context {
        Context::Config => {
            let mut config_comp = ConfigPopupComponent::new();
            config_comp.on_focus();
            config_comp.render(
                frame,
                area,
                &mut crate::ui::components::config_popup::ConfigPopupState {
                    config: &app.config,
                    visible: true,
                },
                &app.theme,
            );
        }
        Context::Help => {
            let mut help_comp = HelpPopupComponent::new();
            help_comp.on_focus();
            help_comp.render(
                frame,
                area,
                &mut crate::ui::components::help_popup::HelpPopupState { visible: true },
                &app.theme,
            );
        }
        _ => {}
    }
}
