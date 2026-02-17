//! Query interface for the [`arXiv API`]
//!
//! This module provide the necessary tool to query the [`arXiv API`].
//! `http://export.arxiv.org/api/query?{parameters}`
//!
//! The parameters are separated with a `&` in the construction of the url. Here are the one
//! useable in this module:
//! - `search_query`: Search query to find articles.
//! - `id_list`: A list of IDs to search from.
//! - `start` and `max_results`: Allow downloading chunks of data.
//! - `sortBy`: Specifies how to sort the retrieved arXiv entries.
//! - `sortOrder`: Defines the sorting order of the entries ("ascending" or "descending").
//!
//! Here is an example of a search query:
//! http://export.arxiv.org/api/query?search_query=ti:"electron thermal conductivity"&sortBy=lastUpdatedDate&sortOrder=ascending
//!
//! For more in-depth documentation, look at the [`arXiv API`] user manual.
//!
//! [`arXiv API`] : https://info.arxiv.org/help/api/user-manual.html

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Display;
use thiserror::Error;

const ARXIV_QUERY_BASE_URL: &str = "https://export.arxiv.org/api/query?";

// --- Construct the search query ---

/// Specifies different query options for searching the arXiv archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchQuery {
    /// Search for articles by title.
    Title(String),
    /// Search for articles by author name.
    Author(String),
    /// Search for articles containing keywords in the abstract.
    Abstract(String),
    /// Search for articles containing keywords in the comment field (less common).
    Comment(String),
    /// Search for articles by journal reference information.
    JournalReference(String),
    /// Search for articles within a specific arXiv category (e.g., "cs.AI").
    Category(String),
    /// Search for articles by report number (usually for internal arXiv purposes).
    ReportNumber(String),
    /// Search for articles using a general query string across all fields.
    /// Use with caution as it might lead to unexpected results due to potential broad matches.
    All(String),
}

impl SearchQuery {
    /// Retrieves the corresponding category code for a given `SearchQuery` variant.
    ///
    /// This function is used internally to map each `SearchQuery` variant (e.g.,
    /// `Title`, `Author`) to its corresponding category code required by the
    /// arXiv API ("ti", "au"). The returned string can be used to construct the final query
    /// string.
    fn category(&self) -> &'static str {
        match self {
            Self::Title(_) => "ti",
            Self::Author(_) => "au",
            Self::Abstract(_) => "abs",
            Self::Comment(_) => "cm",
            Self::JournalReference(_) => "jr",
            Self::Category(_) => "cat",
            Self::ReportNumber(_) => "rn",
            Self::All(_) => "all",
        }
    }
}

impl Display for SearchQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Title(term) => write!(f, "{term}"),
            Self::Author(term) => write!(f, "{term}"),
            Self::Abstract(term) => write!(f, "{term}"),
            Self::Comment(term) => write!(f, "{term}"),
            Self::JournalReference(term) => write!(f, "{term}"),
            Self::Category(term) => write!(f, "{term}"),
            Self::ReportNumber(term) => write!(f, "{term}"),
            Self::All(term) => write!(f, "{term}"),
        }
    }
}

/// Groups and joins search queries for constructing a well-formatted arXiv API query string.
///
/// This function takes a slice of `SeqrchQuery` structs and groups them by their
/// category. It then joins the queries within each category using `+AND+` and
/// combines the category groups with `&` to create a single, valid query string
/// suitable for the arXiv API.
///
/// The function utilizes a `BTreeMap` to ensure a deterministic output order
/// for the categories and their joined queries.
fn group_and_join_queries(search_queries: &[SearchQuery]) -> String {
    let mut grouped_queries: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();

    for query in search_queries {
        grouped_queries
            .entry(query.category())
            .or_default()
            .push(format!("{query}"));
    }

    let mut joined_query: Vec<String> = Vec::new();
    for (category, category_queries) in grouped_queries.iter_mut() {
        let mut category_query = format!("{category}:");
        category_query.push_str(&category_queries.join("+AND+"));
        joined_query.push(category_query);
    }
    joined_query.join("&")
}

// --- Option for the query ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortBy {
    Relevance,
    LastUpdatedDate,
    SubmittedDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
                Self::Relevance => "relevance",
                Self::LastUpdatedDate => "lastUpdatedDate",
                Self::SubmittedDate => "submittedDate",
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
                Self::Ascending => "ascending",
                Self::Descending => "descending",
            }
        )
    }
}

/// Constructs a well-formatted search query string for the arXiv API.
///
/// This function takes various optional parameters for constructing a complete search query
/// string that can be appended to the arXiv API base URL. It handles parameters like:
///
/// - `search_queries`: An optional slice of `SearchQuery` structs representing the search criteria.
/// - `start_index`: An optional integer specifying the starting index for result retrieval (pagination).
/// - `max_results`: An optional integer specifying the maximum number of results to retrieve.
/// - `sort_by`: An optional `SortBy` enum specifying how to sort the retrieved entries.
/// - `sort_order`: An optional `SortOrder` enum specifying the sorting order (ascending or descending).
///
/// The function utilizes the `group_and_join_queries` function to format the `search_queries`
/// if provided. It then combines all parameters with appropriate separators (`=`, `&`) to form
/// a valid and complete query string.
///
/// If no search parameters are provided, an empty string is returned.
pub fn get_search_query(
    search_queries: Option<&[SearchQuery]>,
    start_index: Option<i32>,
    max_results: Option<i32>,
    sort_by: Option<SortBy>,
    sort_order: Option<SortOrder>,
) -> String {
    let mut query: Vec<String> = Vec::new();

    if let Some(search_queries) = search_queries {
        query.push(format!(
            "search_query={}",
            group_and_join_queries(search_queries)
        ));
    }

    if let Some(start) = start_index {
        query.push(format!("start={start}"))
    }

    if let Some(max_res) = max_results {
        query.push(format!("max_results={max_res}"))
    }

    if let Some(sort_by) = sort_by {
        query.push(format!("sortBy={sort_by}"))
    }
    if let Some(sort_order) = sort_order {
        query.push(format!("sortOrder={sort_order}"))
    }

    if query.is_empty() {
        String::new()
    } else {
        query.join("&").to_string()
    }
}

/// Constructs the complete URL for querying the arXiv archive.
///
/// This function takes various optional parameters for constructing a complete search query
/// string that can be appended to the arXiv API base URL. It handles parameters like:
///
/// - `search_queries`: An optional slice of `SearchQuery` structs representing the search criteria.
/// - `start_index`: An optional integer specifying the starting index for result retrieval (pagination).
/// - `max_results`: An optional integer specifying the maximum number of results to retrieve.
/// - `sort_by`: An optional `SortBy` enum specifying how to sort the retrieved entries.
/// - `sort_order`: An optional `SortOrder` enum specifying the sorting order (ascending or descending).
///
/// This function construct the final URL for querying the arXiv archive. It accomplishes this by:
///
/// 1. **Calling `get_search_query`**: It calls the `get_search_query` function with the provided
///    parameters to generate the formatted search query string.
/// 2. **Appending Search Query**: It then appends the generated search query string to the
///    predefined `ARXIV_QUERY_BASE_URL` constant, which specifies the base URL for arXiv API queries.
///
/// By combining these steps, this function creates a complete and valid URL ready to be used
/// for fetching data from the arXiv archive.
pub fn get_query_url(
    search_queries: Option<&[SearchQuery]>,
    start_index: Option<i32>,
    max_results: Option<i32>,
    sort_by: Option<SortBy>,
    sort_order: Option<SortOrder>,
) -> String {
    let search_query = get_search_query(
        search_queries,
        start_index,
        max_results,
        sort_by,
        sort_order,
    );
    format!("{ARXIV_QUERY_BASE_URL}{search_query}")
}

#[derive(Debug, Error)]
pub enum ArxivQueryError {
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),
}

/// Query arXiv with the query url.
pub fn query_arxiv(
    search_queries: Option<&[SearchQuery]>,
    start_index: Option<i32>,
    max_results: Option<i32>,
    sort_by: Option<SortBy>,
    sort_order: Option<SortOrder>,
) -> Result<String, ArxivQueryError> {
    let query_str = get_query_url(
        search_queries,
        start_index,
        max_results,
        sort_by,
        sort_order,
    );
    let ua = format!(
        "arxivlens/{} (+https://github.com/AlMrvn/arxivlens)",
        env!("CARGO_PKG_VERSION")
    );
    let client = reqwest::blocking::Client::builder()
        .user_agent(ua)
        .build()
        .map_err(ArxivQueryError::NetworkError)?;
    let response = client
        .get(query_str)
        .send()
        .map_err(ArxivQueryError::NetworkError)?;

    let text = response.text()?;

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- Testing the construction of the query url -----
    #[test]
    fn test_get_search_query_basic() {
        let url = get_query_url(None, None, None, None, None);
        assert_eq!(url, ARXIV_QUERY_BASE_URL);
    }

    #[test]
    fn test_get_search_query_category() {
        let url = get_query_url(
            Some(&[SearchQuery::Category("cs.AI".to_string())]),
            None,
            None,
            None,
            None,
        );
        assert_eq!(
            url,
            format!("{}search_query=cat:cs.AI", ARXIV_QUERY_BASE_URL)
        );
    }

    #[test]
    fn test_get_search_query_author() {
        let url = get_query_url(
            Some(&[SearchQuery::Author("Albert Einstein".to_string())]),
            None,
            None,
            None,
            None,
        );
        assert_eq!(
            url,
            format!("{}search_query=au:Albert Einstein", ARXIV_QUERY_BASE_URL)
        );
    }

    #[test]
    fn test_get_search_query_all_params() {
        let url = get_query_url(
            Some(&[
                SearchQuery::Author("Jane Doe".to_string()),
                SearchQuery::Category("stat.ML".to_string()),
            ]),
            Some(10),
            Some(50),
            Some(SortBy::LastUpdatedDate),
            Some(SortOrder::Descending),
        );
        assert_eq!(
        url,
        format!(
          "{}search_query=au:Jane Doe&cat:stat.ML&start=10&max_results=50&sortBy=lastUpdatedDate&sortOrder=descending",
          ARXIV_QUERY_BASE_URL
        )
      );
    }

    #[test]
    fn test_group_and_join_queries() {
        // Sample list of SearchQuery structs
        let queries = vec![
            SearchQuery::Category("quant-ph".to_string()),
            SearchQuery::Author("Doe".to_string()),
            SearchQuery::Title("Holes".to_string()),
            SearchQuery::Abstract("Entanglement".to_string()),
        ];

        // Expected encoded query string
        let expected_query = "abs:Entanglement&au:Doe&cat:quant-ph&ti:Holes";

        // Test the function and compare with expected result
        let encoded_query = group_and_join_queries(&queries);
        assert_eq!(encoded_query, expected_query);
    }

    #[test]
    fn test_group_and_join_queries_multiple_same_category() {
        // Sample list of SearchQuery structs
        let queries = vec![
            SearchQuery::Title("Quantum Mechanics".to_string()),
            SearchQuery::Author("John Doe".to_string()),
            SearchQuery::Title("Black Holes".to_string()),
            SearchQuery::Title("Relativity".to_string()),
            SearchQuery::Abstract("Entanglement".to_string()),
        ];

        // Expected encoded query string
        let expected_query =
            "abs:Entanglement&au:John Doe&ti:Quantum Mechanics+AND+Black Holes+AND+Relativity";

        // Test the function
        let encoded_query = group_and_join_queries(&queries);

        // Assert encoded query matches expectation
        assert_eq!(encoded_query, expected_query);
    }
}
