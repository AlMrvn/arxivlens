//! Parsing arXiv entry from the [`arXiv API`]
//!
//! This module prove the tools to construct the list ofentry (or manuscripts) out of the
//! XML string obtained from the query of the arXiv API.

use minidom::Element;
use std::error::Error;

use crate::search_highlight::search_patterns;

const ENTRY_NS: &str = "http://www.w3.org/2005/Atom";

#[derive(Debug, Default, PartialEq)]
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

/// Storing the result of the arxiv query
#[derive(Debug, Default, PartialEq)]
pub struct ArxivQueryResult {
    pub updated: String,
    pub articles: Vec<ArxivEntry>,
}

impl ArxivQueryResult {
    pub fn from_xml_content(content: &str) -> Self {
        let root: Element = content.parse().unwrap();

        // Find the updated
        let query_update = root.get_child("updated", ENTRY_NS).unwrap().text();

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
                let authors = match extract_authors(child) {
                    Ok(auths) => auths,
                    Err(_) => vec!["Error while parsing authors names".to_string()],
                };

                // Only add the new entry, ie published == updated
                if updated.as_str() == published.as_str() {
                    articles.push(ArxivEntry::new(
                        title.replace("\n ", "").to_owned(), // arxiv has this formatting
                        authors.to_owned(),
                        summary.replace('\n', " ").to_owned(),
                        id.to_owned(),
                        updated.to_owned(),
                        published.to_owned(),
                    ));
                }
            }
        }
        let articles = articles;
        Self {
            updated: query_update,
            articles,
        }
    }
    pub fn from_query(query: String) -> Self {
        let query_response = match reqwest::blocking::get(query) {
            Ok(content) => content,
            Err(error) => panic!("Problem while querying arXiv: {error:?}"),
        };
        let xml_content = query_response.text().unwrap_or_else(|e| {
            eprintln!("Request failed: {}", e);
            std::process::exit(1);
        });
        ArxivQueryResult::from_xml_content(&xml_content)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

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
    fn test_empty_author_list() -> Result<(), Box<dyn Error>> {
        let author_element = Element::from_str(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <feed xmlns="http://www.w3.org/2005/Atom">
            </feed>"#,
        );

        let extracted_authors = extract_authors(&author_element?)?;
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

        let actual_result = ArxivQueryResult::from_xml_content(&xml_content);

        assert_eq!(expected_result, actual_result);

        Ok(())
    }
}
