use arxivlens::app::{App, AppResult};
use arxivlens::arxiv::{get_query_url, ArxivQueryResult, SearchQuery, SortBy, SortOrder};
use arxivlens::config;
use arxivlens::event::{Event, EventHandler};
use arxivlens::handler::handle_key_events;
use arxivlens::tui::Tui;
use arxivlens::ui::Theme;
use clap::Parser;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

/// Default values for the query:
const DEFAULT_START_INDEX: i32 = 0;
const DEFAULT_MAX_RESULTS: i32 = 200;
const DEFAULT_SORT_ORDER: SortOrder = SortOrder::Descending;
const DEFAULT_SORT_BY: SortBy = SortBy::SubmittedDate;

/// Terminal User Interface to explore arXiv
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the author to search for in arXiv entries
    #[arg(short, long, default_value = None)]
    author: Option<String>,

    /// ArXiv category to search (e.g., "quant-ph", "cs.AI")
    #[arg(short, long, default_value = None)]
    category: Option<String>,
}

fn main() -> AppResult<()> {
    // --- Construct the arXiv query with the user args ---
    let args = Args::parse();
    let config =
        config::Config::load().map_err(|e| format!("Failed to load configuration: {}", e))?;

    // TODO: Get the them out of the config:
    let theme = Theme::default();

    //
    let mut queries: Vec<SearchQuery> = Vec::new();

    if let Some(author) = &args.author {
        queries.push(SearchQuery::Author(author.to_string()))
    }
    if let Some(category) = &args.category {
        queries.push(SearchQuery::Category(category.to_string()))
    } else {
        queries.push(SearchQuery::Category(config.query.category.clone()))
    }

    // --- Query the arxiv API ---
    let query = get_query_url(
        Some(&queries),
        Some(DEFAULT_START_INDEX),
        Some(DEFAULT_MAX_RESULTS),
        Some(DEFAULT_SORT_BY),
        Some(DEFAULT_SORT_ORDER),
    );
    let query_result = ArxivQueryResult::from_query(query);

    // Create a longer-lived value for the highlight config
    let highlight_config = config.highlight.clone();

    // Create an application.
    let mut app = App::new(&query_result, &highlight_config, theme, config);

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new();
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
