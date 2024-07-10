use minidom::Element;
use std::error::Error;
use url::Url;

const ENTRY_NS: &'static str = "http://www.w3.org/2005/Atom";

fn get_search_query_str() -> Result<String, Box<dyn Error>> {
    let s = String::from("cat:quant-ph");
    let start_index: u32 = 0;
    let base_url = Url::parse("http://export.arxiv.org/api/query?")?;
    let mut search_url = base_url;
    search_url
        .query_pairs_mut()
        .append_pair("search_query", &s.to_string());
    search_url
        .query_pairs_mut()
        .append_pair("sortBy", "lastUpdatedDate");
    search_url
        .query_pairs_mut()
        .append_pair("start", &start_index.to_string());
    search_url.query_pairs_mut().append_pair("max_results", "1");

    Ok(search_url.to_string())
}

fn query_arxiv() -> Result<String, Box<dyn Error>> {
    let search_query_str = "http://export.arxiv.org/api/query?search_query=cat%3Aquant-ph&sortBy=lastUpdatedDate&start=0&max_results=20";
    Ok(reqwest::blocking::get(search_query_str)?.text()?)
}

fn parse_arxiv_entries(content: &str) -> Result<Vec<Entry>, Box<dyn Error>> {
    let root: Element = content.parse().unwrap();
    let mut articles: Vec<Entry> = Vec::new();

    for child in root.children() {
        if child.is("entry", ENTRY_NS) {
            let title = child.get_child("title", ENTRY_NS).unwrap().text();
            let id = child.get_child("id", ENTRY_NS).unwrap().text();

            let mut names: Vec<String> = Vec::new();

            // THe issue is coming from here where this will just return the first author.
            let authors = child.get_child("author", ENTRY_NS);
            for author in authors {
                let name = author.get_child("name", ENTRY_NS).unwrap().text();
                names.push(name.clone());
                println!("{}", name);
            }
            articles.push(Entry {
                title: title,
                id: id,
                first_author: names.first().unwrap().to_string(),
                last_author: names.last().unwrap().to_string(),
            });
        }
    }
    Ok(articles)
}

#[derive(Debug)]
pub struct Entry {
    title: String,
    id: String,
    first_author: String,
    last_author: String,
}

// fn main() -> Result<(), Box<dyn Error>> {
//     // Getting the articles from arXiv:
//     let content = query_arxiv()?;
//     println!("{}", content);
//
//     // Parsing the XML file that we got:
//     // We are gonna use the Struct Entry and add them to a Vec of them.
//     let articles = parse_arxiv_entries(&content)?;
//
//     // Formatting to print the result
//     let mut table = Table::new(&articles);
//     println!("{}", table.with(Style::sharp()));
//
//     Ok(())
// }
