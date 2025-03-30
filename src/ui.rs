pub mod config_popup;
pub mod detail;
pub mod list;
pub mod style;

pub use config_popup::ConfigPopup;
pub use detail::ArticleDetails;
pub use list::ArticleFeed;
pub use style::Theme;

fn option_vec_to_option_slice(option_vec: &Option<Vec<String>>) -> Option<Vec<&str>> {
    let binding = option_vec
        .as_deref()
        .map(|v| v.iter().map(String::as_str).collect::<Vec<&str>>());
    binding
}
