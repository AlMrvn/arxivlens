//! Query interface for the [`arXiv API`]
//!
//! This module provide the necessary tool to query (using reqwest) and pasrse
//! the arxiv XML file and cast it into a struct found in arxiv_entry module.
//!
//! Context for the API:
//! The parameters for each of the API has a base url that reads:
//! `http://export.arxiv.org/api/query?{parameters}`
//!
//! The parameters are separated with a `&` in the construction of the url. Here are the one
//! useable in this module:
//! - search_query : Search query to find article,
//! - id_list: a list of id to search from,
//! - start and max_results allows to download chunk of the data.
//!
//!
//! [`arXiv API`] : https://info.arxiv.org/help/api/user-manual.html

use std::error::Error;
use std::fmt::Display;

const ARXIV_QUERY_BASE_URL: &'static str = "http://export.arxiv.org/api/query?";

// ----- Enum for the option of the arXiv query -----
pub enum SortBy {
    Relevance,
    LastUpdatedDate,
    SubmittedDate,
}

pub enum SortOrder {
    Ascending,
    Descending,
}

impl Display for SortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SortBy::Relevance => "relevance",
                SortBy::LastUpdatedDate => "lastUpdatedDate",
                SortBy::SubmittedDate => "submittedDate",
            }
        )
    }
}

impl Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SortOrder::Ascending => "ascending",
                SortOrder::Descending => "descending",
            }
        )
    }
}

/// Constructing the search query string to add to the base url.
///
fn get_search_query(
    category: Option<&str>,
    author: Option<&str>,
    start_index: Option<usize>,
    max_results: Option<usize>,
    sort_by: Option<SortBy>,
    sort_order: Option<SortOrder>,
) -> String {
    let mut search_query: Vec<String> = Vec::new();

    if let Some(cat) = category {
        search_query.push(format!("cat:{}", cat));
    }

    if let Some(auth) = author {
        search_query.push(format!("au:{}", auth));
    }

    if let Some(start) = start_index {
        search_query.push(format!("start={}", start))
    }

    if let Some(max_res) = max_results {
        search_query.push(format!("max_results={}", max_res))
    }

    if let Some(sort_by) = sort_by {
        search_query.push(format!("sortBy={}", sort_by))
    }
    if let Some(sort_order) = sort_order {
        search_query.push(format!("sortOrder={}", sort_order))
    }

    // If the string search str is empty, we return an empty string:
    if search_query.len() == 0 {
        String::new()
    // Else, we return the query with each type of query separated with an &
    } else {
        format!("search_query={}", search_query.join("&"))
    }
}

/// Constructs a URL string for querying the arXiv archive based on search parameters.
///
/// This function takes various optional parameters to construct a URL string suitable
/// for querying the arXiv archive.
///
/// Parameters:
///
/// * `category`: An optional arXiv category to search within.
/// * `author`: An optional reference to a string slice representing the author's name.
/// * `start_index`: An optional `usize` representing the starting index for search results.
/// * `max_results`: An optional `usize` representing the maximum number of results to return.
///     * If not provided, a default value (`DEFAULT_MAX_RESULTS`) is used.
/// * `sort_by`: An optional `SortBy` enum variant specifying the sorting criteria for results.
///     * Defaults to `DEFAULT_SORT_BY` if not provided.
/// * `sort_order`: An optional `SortOrder` enum variant specifying the sorting order (ascending or descending).
///     * Defaults to the default order associated with the chosen `sort_by` option.
fn get_query_url(
    category: Option<&str>,
    author: Option<&str>,
    start_index: Option<usize>,
    max_results: Option<usize>,
    sort_by: Option<SortBy>,
    sort_order: Option<SortOrder>,
) -> String {
    let search_query = get_search_query(
        category,
        author,
        start_index,
        max_results,
        sort_by,
        sort_order,
    );
    format!("{}{}", ARXIV_QUERY_BASE_URL, search_query)
}

pub fn query_arxiv() -> Result<String, Box<dyn Error>> {
    let query_str = get_query_url(
        Some("quant-ph"),
        None,
        Some(0),
        Some(200),
        Some(SortBy::SubmittedDate),
        Some(SortOrder::Descending),
    );
    Ok(reqwest::blocking::get(query_str)?.text()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- Testing the construction of the query url -----
    #[test]
    fn test_get_search_query_basic() {
        let url = get_query_url(None, None, None, None, None, None);
        assert_eq!(url, ARXIV_QUERY_BASE_URL);
    }

    #[test]
    fn test_get_search_query_category() {
        let url = get_query_url(Some("cs.AI"), None, None, None, None, None);
        assert_eq!(
            url,
            format!("{}search_query=cat:cs.AI", ARXIV_QUERY_BASE_URL)
        );
    }

    #[test]
    fn test_get_search_query_author() {
        let url = get_query_url(None, Some("Albert Einstein"), None, None, None, None);
        assert_eq!(
            url,
            format!("{}search_query=au:Albert Einstein", ARXIV_QUERY_BASE_URL)
        );
    }

    #[test]
    fn test_get_search_query_all_params() {
        let url = get_query_url(
            Some("stat.ML"),
            Some("Jane Doe"),
            Some(10),
            Some(50),
            Some(SortBy::LastUpdatedDate),
            Some(SortOrder::Descending),
        );
        assert_eq!(
        url,
        format!(
          "{}search_query=cat:stat.ML&au:Jane Doe&start=10&max_results=50&sortBy=lastUpdatedDate&sortOrder=descending",
          ARXIV_QUERY_BASE_URL
        )
      );
    }
}
