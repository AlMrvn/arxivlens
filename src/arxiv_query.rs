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

use crate::arxiv_entry::ArxivEntry;
use minidom::Element;
use std::fmt::Display;
use std::{error::Error, str::FromStr};

const ENTRY_NS: &'static str = "http://www.w3.org/2005/Atom";
const ARXIV_QUERY_BASE_URL: &'static str = "http://export.arxiv.org/api/query?";
const DEFAULT_MAX_RESULTS: usize = 200;
const DEFAULT_SORT_BY: SortBy = SortBy::SubmittedDate;
const DEFAULT_SORT_ORDER: SortOrder = SortOrder::Descending;

enum SortBy {
    Relevance,
    LastUpdatedDate,
    SubmittedDate,
}

enum SortOrder {
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

fn get_search_str(
    category: Option<&str>,
    author: Option<&str>,
    start_index: Option<usize>,
    max_results: Option<usize>,
    sort_by: Option<SortBy>,
    sort_order: Option<SortOrder>,
) -> String {
    let mut search_str = String::new();

    if let Some(cat) = category {
        search_str.push_str(&format!("&cat:{}", cat));
    }

    if let Some(auth) = author {
        search_str.push_str(&format!("&au:{}", auth));
    }

    if let Some(start) = start_index {
        search_str.push_str(&format!("&start={}", start))
    }

    if let Some(max_res) = max_results {
        search_str.push_str(&format!("&max_results={}", max_res))
    }

    if let Some(sort_by) = sort_by {
        search_str.push_str(&format!("&sortBy={}", sort_by))
    }
    if let Some(sort_order) = sort_order {
        search_str.push_str(&format!("&sortOrder={}", sort_order))
    }

    // If the string search str is empty, we return an empty string:
    if search_str.len() == 0 {
        search_str
    } else {
        format!("search_query={}", search_str)
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
fn get_query_url_str(
    category: Option<&str>,
    author: Option<&str>,
    start_index: Option<usize>,
    max_results: Option<usize>,
    sort_by: Option<SortBy>,
    sort_order: Option<SortOrder>,
) -> String {
    let search_str = get_search_str(
        category,
        author,
        start_index,
        max_results,
        sort_by,
        sort_order,
    );
    format!("{}{}", ARXIV_QUERY_BASE_URL, search_str)
}

#[test]
fn test_get_search_str_basic() {
    let url = get_query_url_str(None, None, None, None, None, None);
    assert_eq!(url, ARXIV_QUERY_BASE_URL);
}

#[test]
fn test_get_search_str_category() {
    let url = get_query_url_str(Some("cs.AI"), None, None, None, None, None);
    assert_eq!(
        url,
        format!("{}search_query=&cat:cs.AI", ARXIV_QUERY_BASE_URL)
    );
}

#[test]
fn test_get_search_str_author() {
    let url = get_query_url_str(None, Some("Albert Einstein"), None, None, None, None);
    assert_eq!(
        url,
        format!("{}search_query=&au:Albert Einstein", ARXIV_QUERY_BASE_URL)
    );
}

#[test]
fn test_get_search_str_all_params() {
    let url = get_query_url_str(
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
      "{}search_query=&cat:stat.ML&au:Jane Doe&start=10&max_results=50&sortBy=lastUpdatedDate&sortOrder=descending",
      ARXIV_QUERY_BASE_URL
    )
  );
}

/// Constructs a URL string for querying arXiv based on search parameters.
fn get_query_str(
    category: &str,
    start_index: usize,
    max_results: usize,
) -> Result<String, Box<dyn Error>> {
    let query_string = format!(
        "search_query={}&sortBy=lastUpdatedDate&start={}&max_results={}",
        category, start_index, max_results
    );

    Ok(format!("{}{}", ARXIV_QUERY_BASE_URL, query_string))
}

#[test]
fn test_get_query_str() -> Result<(), Box<dyn Error>> {
    let expected_url = "http://export.arxiv.org/api/query?search_query=cat:quant-ph&sortBy=lastUpdatedDate&start=0&max_results=200";
    let actual_url = get_query_str("cat:quant-ph", 0, 200)?;

    assert_eq!(expected_url, actual_url);

    Ok(())
}

pub fn query_arxiv() -> Result<String, Box<dyn Error>> {
    let query_str = get_query_str("cat:quant-ph", 0, 200)?;
    Ok(reqwest::blocking::get(query_str)?.text()?)
}

pub fn parse_arxiv_entries(content: &str) -> Result<Vec<ArxivEntry>, Box<dyn Error>> {
    let root: Element = content.parse().unwrap();
    let mut articles: Vec<ArxivEntry> = Vec::new();

    for child in root.children() {
        if child.is("entry", ENTRY_NS) {
            // Extract the main information
            let title = child.get_child("title", ENTRY_NS).unwrap().text();
            let id = child.get_child("id", ENTRY_NS).unwrap().text();
            let summary = child.get_child("summary", ENTRY_NS).unwrap().text();
            let updated = child.get_child("updated", ENTRY_NS).unwrap().text();
            let published = child.get_child("published", ENTRY_NS).unwrap().text();

            // Extract the authors which have one more depth.
            let authors = extract_authors(child)?;

            // Only add the new entry, ie published == updated
            match updated.as_str() == published.as_str() {
                true => articles.push(ArxivEntry {
                    title,
                    authors,
                    summary,
                    id,
                    updated,
                    published,
                }),
                _ => (),
            }
        }
    }
    Ok(articles)
}

/// Helper function to extract the authors
fn extract_authors(entry: &Element) -> Result<Vec<String>, Box<dyn Error>> {
    let mut names: Vec<String> = Vec::new();

    // Since there are several child with the same name, we iterate over all of them:
    for child in entry.children() {
        if child.is("author", ENTRY_NS) {
            let name = child.get_child("name", ENTRY_NS).unwrap().text();
            names.push(name)
        }
    }

    Ok(names)
}

#[test]
fn test_extract_authors() -> Result<(), Box<dyn Error>> {
    let author_element = Element::from_str(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <author>
    <name>Author Name 1</name>
   </author>
   <author>
    <name>Author Name 2, Second</name>
  </author>
  </feed>
  "#,
    );

    let expected_authors = vec![
        String::from("Author Name 1"),
        String::from("Author Name 2, Second"),
    ];
    let extracted_authors = extract_authors(&author_element?)?;

    assert_eq!(expected_authors, extracted_authors);

    Ok(())
}

#[test]
fn test_parse_arxiv_entries() -> Result<(), Box<dyn Error>> {
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <link href="http://arxiv.org/api/query?search_query=fake%3Atopic&amp;id_list=&amp;start=0&amp;max_results=20" rel="self" type="application/atom+xml"/>
  <title type="html">ArXiv Query: search_query=fake:topic&amp;id_list=&amp;start=0&amp;max_results=20</title>
  <id>http://arxiv.org/api/FAKESAMPLEID</id>
  <updated>2024-07-09T20:00:00Z</updated>
  <opensearch:totalResults xmlns:opensearch="http://a9.com/-/spec/opensearch/1.1/">10</opensearch:totalResults>
  <opensearch:startIndex xmlns:opensearch="http://a9.com/-/spec/opensearch/1.1/">0</opensearch:startIndex>
  <opensearch:itemsPerPage xmlns:opensearch="http://a9.com/-/spec/opensearch/1.1/">20</opensearch:itemsPerPage>
  <entry>
    <id>http://arxiv.org/abs/9876.54321</id>
    <updated>2023-12-31T23:59:59Z</updated>
    <published>2023-12-31T23:59:59Z</published>
    <title>Sample Title 1</title>
    <summary>This is a summary for the first fake entry used for testing purposes.</summary>
    <author>
      <name>Author One</name>
    </author>
    <author>
      <name>Author Two</name>
    </author>
  </entry>
  <entry>
    <id>http://arxiv.org/abs/1212.34567</id>
    <updated>2024-01-01T00:00:00Z</updated>
    <published>2024-01-01T00:00:00Z</published>
    <title>Sample Title 2</title>
    <summary>This is a sample summary for the second entry.</summary>
    <author>
      <name>Author Three</name>
    </author>
  </entry>
</feed>  "#
    .to_string();

    let expected_entries = vec![
        ArxivEntry {
            title: String::from("Sample Title 1"),
            authors: [String::from("Author One"), String::from("Author Two")].to_vec(),
            summary: String::from(
                "This is a summary for the first fake entry used for testing purposes.",
            ),
            id: String::from("http://arxiv.org/abs/9876.54321"),
            updated: String::from("2023-12-31T23:59:59Z"),
            published: String::from("2023-12-31T23:59:59Z"),
        },
        ArxivEntry {
            title: String::from("Sample Title 2"),
            authors: [String::from("Author Three")].to_vec(),
            summary: String::from("This is a sample summary for the second entry."),
            id: String::from("http://arxiv.org/abs/1212.34567"),
            updated: String::from("2024-01-01T00:00:00Z"),
            published: String::from("2024-01-01T00:00:00Z"),
        },
    ];

    let parsed_entries = parse_arxiv_entries(&xml_content)?;

    assert_eq!(expected_entries, parsed_entries);

    Ok(())
}
