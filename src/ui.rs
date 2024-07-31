mod detail;
mod list;
mod style;

pub use detail::*;
pub use list::*;
pub use style::*;


fn option_vec_to_option_slice(option_vec: &Option<Vec<String>>) -> Option<Vec<&str>> {
    let binding = option_vec
        .as_deref()
        .map(|v| v.iter().map(String::as_str).collect::<Vec<&str>>());
    binding
}