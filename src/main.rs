use arxivlens::app::{App, AppResult, ArxivEntryList};
use arxivlens::arxiv_parsing::parse_arxiv_entries;
use arxivlens::arxiv_query::{query_arxiv, SearchQuery, SortBy, SortOrder};
use arxivlens::event::{Event, EventHandler};
use arxivlens::handler::handle_key_events;
use arxivlens::tui::Tui;
use clap::Parser;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::ListState;
use ratatui::Terminal;
use std::io;

/// Default values for the query:
const DEFAULT_CATEGORY: &str = "quant-ph";
const DEFAULT_START_INDEX: i32 = 0;
const DEFAULT_MAX_RESULTS: i32 = 200;
const DEFAULT_SORT_ORDER: SortOrder = SortOrder::Descending;
const DEFAULT_SORT_BY: SortBy = SortBy::SubmittedDate;

/// Terminal User Interface to explore arXiv
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the author to look
    #[arg(short, long, default_value = None)]
    author: Option<String>,

    /// Number of times to greet
    #[arg(short, long, default_value = DEFAULT_CATEGORY)]
    category: Option<String>,
}

fn main() -> AppResult<()> {
    // --- Construct the arXiv query with the user args ---
    let args = Args::parse();
    let mut queries: Vec<SearchQuery> = Vec::new();

    if let Some(author) = &args.author {
        queries.push(SearchQuery::Author(author.to_string()))
    }
    if let Some(category) = &args.category {
        queries.push(SearchQuery::Category(category.to_string()))
    } else {
        queries.push(SearchQuery::Category(DEFAULT_CATEGORY.to_string()))
    }

    // --- Query the arxiv API ---
    let content = query_arxiv(
        Some(&queries),
        Some(DEFAULT_START_INDEX),
        Some(DEFAULT_MAX_RESULTS),
        Some(DEFAULT_SORT_BY),
        Some(DEFAULT_SORT_ORDER),
    );
    let items = parse_arxiv_entries(&content?)?;
    let state = ListState::default();

    // Create an application.
    let mut app = App {
        running: true,
        arxiv_entries: ArxivEntryList { items, state },
    };

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
