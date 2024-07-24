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

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Display;

const ARXIV_QUERY_BASE_URL: &'static str = "http://export.arxiv.org/api/query?";

// ----- Enum for the option of the arXiv query -----
pub enum ArxivSearchQuery {
    Title(String),
    Author(String),
    Abstract(String),
    Comment(String),
    JournalReference(String),
    Category(String),
    ReportNumber(String),
    All(String),
}

pub enum SortBy {
    Relevance,
    LastUpdatedDate,
    SubmittedDate,
}

pub enum SortOrder {
    Ascending,
    Descending,
}

impl Display for ArxivSearchQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArxivSearchQuery::Title(term) => write!(f, "{}", term),
            ArxivSearchQuery::Author(term) => write!(f, "{}", term),
            ArxivSearchQuery::Abstract(term) => write!(f, "{}", term),
            ArxivSearchQuery::Comment(term) => write!(f, "{}", term),
            ArxivSearchQuery::JournalReference(term) => write!(f, "{}", term),
            ArxivSearchQuery::Category(term) => write!(f, "{}", term),
            ArxivSearchQuery::ReportNumber(term) => write!(f, "{}", term),
            ArxivSearchQuery::All(term) => write!(f, "{}", term),
        }
    }
}

impl ArxivSearchQuery {
    fn category(&self) -> &'static str {
        match self {
            ArxivSearchQuery::Title(_) => "ti",
            ArxivSearchQuery::Author(_) => "au",
            ArxivSearchQuery::Abstract(_) => "abs",
            ArxivSearchQuery::Comment(_) => "cm",
            ArxivSearchQuery::JournalReference(_) => "jr",
            ArxivSearchQuery::Category(_) => "cat",
            ArxivSearchQuery::ReportNumber(_) => "rn",
            ArxivSearchQuery::All(_) => "all",
        }
    }
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

fn group_and_join_queries(queries: &[ArxivSearchQuery]) -> String {
    // Using a BTreeMap to have an ordering of the keys and hence deterministic output.
    let mut grouped_queries: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();

    for query in queries {
        grouped_queries
            .entry(query.category())
            .or_insert(Vec::new())
            .push(format!("{}", query));
    }

    let mut joined_query: Vec<String> = Vec::new();
    for (category, category_queries) in grouped_queries.iter_mut() {
        let mut category_query = format!("{}:", category);
        category_query.push_str(&category_queries.join("+AND+"));
        joined_query.push(category_query);
    }
    joined_query.join("&")
}

/// Constructing the search query string to add to the base url.
fn get_search_query(
    search_queries: Option<&[ArxivSearchQuery]>,
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
        query.push(format!("start={}", start))
    }

    if let Some(max_res) = max_results {
        query.push(format!("max_results={}", max_res))
    }

    if let Some(sort_by) = sort_by {
        query.push(format!("sortBy={}", sort_by))
    }
    if let Some(sort_order) = sort_order {
        query.push(format!("sortOrder={}", sort_order))
    }

    // If the string search str is empty, we return an empty string:
    if query.len() == 0 {
        String::new()
    // Else, we return the query with each type of query separated with an &
    } else {
        format!("{}", query.join("&"))
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
    search_queries: Option<&[ArxivSearchQuery]>,
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
    format!("{}{}", ARXIV_QUERY_BASE_URL, search_query)
}

pub fn query_arxiv(
    search_queries: Option<&[ArxivSearchQuery]>,
    start_index: Option<i32>,
    max_results: Option<i32>,
    sort_by: Option<SortBy>,
    sort_order: Option<SortOrder>,
) -> Result<String, Box<dyn Error>> {
    let query_str = get_query_url(
        search_queries,
        start_index,
        max_results,
        sort_by,
        sort_order,
    );
    println!("{}", query_str);
    Ok(reqwest::blocking::get(query_str)?.text()?)
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
            Some(&[ArxivSearchQuery::Category("cs.AI".to_string())]),
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
            Some(&[ArxivSearchQuery::Author("Albert Einstein".to_string())]),
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
                ArxivSearchQuery::Author("Jane Doe".to_string()),
                ArxivSearchQuery::Category("stat.ML".to_string()),
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
        // Sample list of ArxivSearchQuery structs
        let queries = vec![
            ArxivSearchQuery::Category("quant-ph".to_string()),
            ArxivSearchQuery::Author("Doe".to_string()),
            ArxivSearchQuery::Title("Holes".to_string()),
            ArxivSearchQuery::Abstract("Entanglement".to_string()),
        ];

        // Expected encoded query string
        let expected_query = "abs:Entanglement&au:Doe&cat:quant-ph&ti:Holes";

        // Test the function and compare with expected result
        let encoded_query = group_and_join_queries(&queries);
        assert_eq!(encoded_query, expected_query);
    }

    #[test]
    fn test_group_and_join_queries_multiple_same_category() {
        // Sample list of ArxivSearchQuery structs
        let queries = vec![
            ArxivSearchQuery::Title("Quantum Mechanics".to_string()),
            ArxivSearchQuery::Author("John Doe".to_string()),
            ArxivSearchQuery::Title("Black Holes".to_string()),
            ArxivSearchQuery::Title("Relativity".to_string()),
            ArxivSearchQuery::Abstract("Entanglement".to_string()),
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
