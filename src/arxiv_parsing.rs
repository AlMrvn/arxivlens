//! Parsing arXiv entry from the [`arXiv API`]
//!
//! This module prove the tools to construct the list ofentry (or manuscripts) out of the
//! XML string obtained from the query of the arXiv API.

use minidom::Element;
use std::error::Error;

const ENTRY_NS: &'static str = "http://www.w3.org/2005/Atom";

#[derive(Debug, Default, PartialEq)]
pub struct ArxivEntry {
    pub title: String,
    pub authors: Vec<String>,
    pub summary: String,
    pub id: String,
    pub updated: String,
    pub published: String,
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
        Self {
            title,
            authors,
            summary,
            id,
            updated,
            published,
        }
    }
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
    // Shadowing to make it immutable
    let articles = articles;
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
}