use crate::arxiv_entry::ArxivEntry;
use minidom::Element;
use std::error::Error;

const ENTRY_NS: &'static str = "http://www.w3.org/2005/Atom";

fn get_search_query_str() -> Result<String, Box<dyn Error>> {
    let category = String::from("cat:quant-ph");
    let start_index = 0;
    let base_url = "http://export.arxiv.org/api/query?";

    let query_string = format!(
        "search_query={}&sortBy=lastUpdatedDate&start={}&max_results={}",
        category, start_index, 20
    );

    Ok(format!("{}{}", base_url, query_string))
}

pub fn query_arxiv() -> Result<String, Box<dyn Error>> {
    let search_query_str = get_search_query_str()?;
    Ok(reqwest::blocking::get(search_query_str)?.text()?)
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

            // Extract the authors which have one more depth.
            let mut names: Vec<String> = Vec::new();

            let authors = child.get_child("author", ENTRY_NS);
            for author in authors {
                let name = author.get_child("name", ENTRY_NS).unwrap().text();
                names.push(name.clone());
                println!("{}", name);
            }
            articles.push(ArxivEntry {
                title,
                author: names.first().unwrap().to_string(),
                summary,
                id,
            });
        }
    }
    Ok(articles)
}
