//! Parsing arXiv entry from the [`arXiv API`]
//!
//! This module prove the tools to construct the list ofentry (or manuscripts) out of the
//! XML string obtained from the query of the arXiv API.

use minidom::Element;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::search_highlight::search_patterns;

const ENTRY_NS: &str = "http://www.w3.org/2005/Atom";

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct ArxivEntry {
    pub title: String,
    pub authors: Vec<String>,
    pub summary: String,
    pub id: String,
    pub updated: String,
    pub published: String,
    all_authors: String,
}

impl ArxivEntry {
    pub fn new(
        title: String,
        authors: Vec<String>,
        summary: String,
        id: String,
        updated: String,
        published: String,
    ) -> Self {
        let all_authors = authors.join(", ");
        Self {
            title,
            authors,
            summary,
            id,
            updated,
            published,
            all_authors,
        }
    }

    pub fn get_all_authors(&self) -> &str {
        &self.all_authors
    }

    pub fn contains_author(&self, author_patterns: Option<&[&str]>) -> bool {
        if let Some(patterns) = author_patterns {
            let matches = search_patterns(&self.all_authors, patterns);
            !matches.is_empty()
        } else {
            false
        }
    }
}

#[derive(Debug, Error)]
pub enum ArxivParsingError {
    #[error("Failed to parse XML content: {0}")]
    XmlParseError(String),
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("XML parsing error: {0}")]
    MinidomError(#[from] minidom::Error),
}

/// Helper function to extract the authors
fn extract_authors(entry: &Element) -> Result<Vec<String>, ArxivParsingError> {
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

/// Storing the result of the arxiv query
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct ArxivQueryResult {
    pub updated: String,
    pub articles: Vec<ArxivEntry>,
}

impl ArxivQueryResult {
    pub fn from_xml_content(content: &str) -> Result<Self, ArxivParsingError> {
        let root: Element = content
            .parse()
            .map_err(|e| ArxivParsingError::XmlParseError(format!("Failed to parse XML: {e}")))?;

        // Find the updated
        let query_update = root
            .get_child("updated", ENTRY_NS)
            .ok_or_else(|| ArxivParsingError::MissingField {
                field: "updated".to_string(),
            })?
            .text();

        let mut articles: Vec<ArxivEntry> = Vec::new();

        for child in root.children() {
            if child.is("entry", ENTRY_NS) {
                // Extract the main information
                let title = child
                    .get_child("title", ENTRY_NS)
                    .ok_or_else(|| ArxivParsingError::MissingField {
                        field: "title".to_string(),
                    })?
                    .text();
                let id = child
                    .get_child("id", ENTRY_NS)
                    .ok_or_else(|| ArxivParsingError::MissingField {
                        field: "id".to_string(),
                    })?
                    .text();
                let summary = child
                    .get_child("summary", ENTRY_NS)
                    .ok_or_else(|| ArxivParsingError::MissingField {
                        field: "summary".to_string(),
                    })?
                    .text();
                let updated = child
                    .get_child("updated", ENTRY_NS)
                    .ok_or_else(|| ArxivParsingError::MissingField {
                        field: "updated".to_string(),
                    })?
                    .text();
                let published = child
                    .get_child("published", ENTRY_NS)
                    .ok_or_else(|| ArxivParsingError::MissingField {
                        field: "published".to_string(),
                    })?
                    .text();

                // Extract the authors which have one more depth.
                let authors = extract_authors(child)?;

                // Only add the new entry, ie published == updated
                if updated.as_str() == published.as_str() {
                    articles.push(ArxivEntry::new(
                        title.replace("\n ", ""), // arxiv has this formatting
                        authors,
                        summary.replace('\n', " "),
                        id,
                        updated,
                        published,
                    ));
                }
            }
        }
        Ok(Self {
            updated: query_update,
            articles,
        })
    }
    pub fn from_query(query: String) -> Result<Self, ArxivParsingError> {
        let query_response = reqwest::blocking::get(query)?;
        let xml_content = query_response.text()?;
        Self::from_xml_content(&xml_content)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::str::FromStr;

    use super::*;

    fn load_fixture(name: &str) -> String {
        let mut fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        fixture_path.push("tests");
        fixture_path.push("fixtures");
        fixture_path.push(name);

        fs::read_to_string(&fixture_path)
            .unwrap_or_else(|_| panic!("Failed to read fixture file: {}", fixture_path.display()))
    }

    #[test]
    fn test_extract_authors() -> Result<(), ArxivParsingError> {
        let xml_content = load_fixture("authors_sample.xml");
        let author_element = Element::from_str(&xml_content)?;

        let expected_authors = vec![
            String::from("Author Name 1"),
            String::from("Author Name 2, Second"),
        ];
        let extracted_authors = extract_authors(&author_element)?;

        assert_eq!(expected_authors, extracted_authors);

        Ok(())
    }

    #[test]
    fn test_empty_author_list() -> Result<(), ArxivParsingError> {
        let xml_content = load_fixture("empty_authors.xml");
        let author_element = Element::from_str(&xml_content)?;

        let extracted_authors = extract_authors(&author_element)?;
        assert!(extracted_authors.is_empty());
        Ok(())
    }

    #[test]
    fn test_malformed_author_xml() {
        let malformed_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
            <feed xmlns="http://www.w3.org/2005/Atom">
              <author>
                <name>Incomplete Author
            </feed>"#;

        let result = Element::from_str(malformed_xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_arxiv_entry_contains_author() {
        let entry = ArxivEntry::new(
            "Test Title".to_string(),
            vec!["John Doe".to_string(), "Jane Smith".to_string()],
            "Test Summary".to_string(),
            "test_id".to_string(),
            "2024-03-28".to_string(),
            "2024-03-28".to_string(),
        );

        // Test exact match
        assert!(entry.contains_author(Some(&["John Doe"])));

        // Test partial match
        assert!(entry.contains_author(Some(&["Doe"])));

        // Test case sensitivity
        assert!(entry.contains_author(Some(&["john doe"])));

        // Test no match
        assert!(!entry.contains_author(Some(&["Albert Einstein"])));

        // Test empty patterns
        assert!(!entry.contains_author(Some(&[])));

        // Test None patterns
        assert!(!entry.contains_author(None));
    }

    #[test]
    fn test_get_all_authors() {
        let entry = ArxivEntry::new(
            "Test Title".to_string(),
            vec!["John Doe".to_string(), "Jane Smith".to_string()],
            "Test Summary".to_string(),
            "test_id".to_string(),
            "2024-03-28".to_string(),
            "2024-03-28".to_string(),
        );

        assert_eq!(entry.get_all_authors(), "John Doe, Jane Smith");
    }

    #[test]
    fn test_parse_arxiv_entries() -> Result<(), ArxivParsingError> {
        let xml_content = load_fixture("sample_arxiv.xml");
        let expected_result = ArxivQueryResult {
            updated: "2024-07-09T20:00:00Z".to_string(),
            articles: vec![
                ArxivEntry {
                    title: String::from("Sample Title 1"),
                    authors: [String::from("Author One"), String::from("Author Two")].to_vec(),
                    summary: String::from(
                        "This is a summary for the first fake entry used for testing purposes.",
                    ),
                    id: String::from("http://arxiv.org/abs/9876.54321"),
                    updated: String::from("2023-12-31T23:59:59Z"),
                    published: String::from("2023-12-31T23:59:59Z"),
                    all_authors: String::from("Author One, Author Two"),
                },
                ArxivEntry {
                    title: String::from("Sample Title 2"),
                    authors: [String::from("Author Three")].to_vec(),
                    summary: String::from("This is a sample summary for the second entry."),
                    id: String::from("http://arxiv.org/abs/1212.34567"),
                    updated: String::from("2024-01-01T00:00:00Z"),
                    published: String::from("2024-01-01T00:00:00Z"),
                    all_authors: String::from("Author Three"),
                },
            ],
        };

        let actual_result = ArxivQueryResult::from_xml_content(&xml_content)?;

        assert_eq!(expected_result, actual_result);

        Ok(())
    }

    #[test]
    fn test_truncated_xml() {
        let xml_content = load_fixture("sample_arxiv.xml");
        // Cut the XML roughly in half, mid-tag to create malformed XML
        let truncated_xml = &xml_content[..xml_content.len() / 2];

        let result = ArxivQueryResult::from_xml_content(truncated_xml);

        // Should return an error, not crash
        assert!(result.is_err());

        // Verify it's specifically a parsing error
        match result {
            Err(ArxivParsingError::XmlParseError(_)) => {
                // This is what we expect
            }
            Err(ArxivParsingError::MinidomError(_)) => {
                // This is also acceptable since minidom might catch it first
            }
            _ => panic!("Expected XML parsing error, got: {:?}", result),
        }
    }
}
