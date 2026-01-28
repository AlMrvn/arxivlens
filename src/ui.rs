pub mod config_popup;
pub mod detail;
pub mod footer;
pub mod help;
pub mod list;
pub mod style;
pub mod utils;

pub use config_popup::ConfigPopup;
pub use detail::ArticleDetails;
pub use footer::render_footer;
pub use help::render_help_popup;
pub use list::ArticleFeed;
pub use style::Theme;

pub fn option_vec_to_option_slice(option_vec: &Option<Vec<String>>) -> Option<Vec<&str>> {
    option_vec
        .as_deref()
        .map(|v| v.iter().map(String::as_str).collect::<Vec<&str>>())
}
