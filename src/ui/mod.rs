pub mod component;
pub mod components;
pub mod config_popup;
pub mod detail;
pub mod footer;
pub mod help;
pub mod list;
pub mod search;
pub mod style;
pub mod testing;
pub mod theme;
pub mod utils;

// Legacy exports (to maintain compatibility during transition)
pub use config_popup::ConfigPopup;
pub use detail::ArticleDetails;
pub use footer::render_footer;
pub use help::render_help_popup;
pub use list::ArticleFeed;
pub use style::Theme as LegacyTheme;
pub use utils::option_vec_to_option_slice;

// New component-based architecture exports
pub use component::{Component, ComponentLayout, LayoutComponent, TestableComponent};
pub use components::{
    ArticleListComponent, ConfigPopupComponent, HelpPopupComponent, SearchBarComponent,
};
pub use testing::GoldenTester;
pub use theme::Theme;
