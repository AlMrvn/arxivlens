mod detail;
mod list;
mod style;

pub use detail::*;
pub use list::*;
pub use style::*;

fn option_vec_to_option_slice<'a>(option_vec: &'a Option<Vec<String>>) -> Option<Vec<&'a str>> {
    let binding = option_vec
        .as_deref()
        .map(|v| v.iter().map(String::as_str).collect::<Vec<&str>>());
    binding
}
