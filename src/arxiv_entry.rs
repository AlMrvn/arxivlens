use ratatui::widgets::ListState;

#[derive(Debug, Default)]
pub struct ArxivEntry {
    pub title: String,
    pub author: String,
    pub summary: String,
}

impl ArxivEntry {
    fn new(title: &str, author: &str, summary: &str) -> Self {
        Self {
            title: title.to_string(),
            author: author.to_string(),
            summary: summary.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ArxivEntryList {
    pub items: Vec<ArxivEntry>,
    pub state: ListState,
}

impl FromIterator<(&'static str, &'static str, &'static str)> for ArxivEntryList {
    fn from_iter<T: IntoIterator<Item = (&'static str, &'static str, &'static str)>>(
        iter: T,
    ) -> Self {
        let items = iter
            .into_iter()
            .map(|(title, author, summary)| ArxivEntry::new(title, author, summary))
            .collect();
        let state = ListState::default();
        Self { items, state }
    }
}
//
// pub const ARXIV_ENTRIES: &[ArxivEntry] = &[
//     ArxivEntry {
//         title: "Alice <alice@example.com>",
//         author: "Hello",
//         summary: "Hi Bob,\nHow are you?\n\nAlice",
//     },
//     ArxivEntry {
//         title: "Bob <bob@example.com>",
//         author: "Re: Hello",
//         summary: "Hi Alice,\nI'm fine, thanks!\n\nBob",
//     },
//     ArxivEntry {
//         title: "Charlie <charlie@example.com>",
//         author: "Re: Hello",
//         summary: "Hi Alice,\nI'm fine, thanks!\n\nCharlie",
//     },
//     ArxivEntry {
//         title: "Dave <dave@example.com>",
//         author: "Re: Hello (STOP REPLYING TO ALL)",
//         summary: "Hi Everyone,\nPlease stop replying to all.\n\nDave",
//     },
//     ArxivEntry {
//         title: "Eve <eve@example.com>",
//         author: "Re: Hello (STOP REPLYING TO ALL)",
//         summary: "Hi Everyone,\nI'm reading all your emails.\n\nEve",
//     },
// ];
