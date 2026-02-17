use arxivlens::app::{App, AppResult};
use arxivlens::arxiv::{query_arxiv, ArxivQueryResult, SearchQuery, SortBy, SortOrder};
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
    // 1. Setup Panic Hook
    setup_panic_hook();

    // 2. Construct the arXiv query with the user args
    let args = Args::parse();
    let config =
        config::Config::load().map_err(|e| format!("Failed to load configuration: {e}"))?;

    // 3. Setup Theme from Config
    let theme = Theme::from_config(&config);

    // 4. Fetch Data from ArXiv API
    let query_result = resolve_query(&args, &config)?;

    // Create a longer-lived value for the highlight config
    let pinned_config = config.pinned.clone();

    // Create an application.
    let mut app = App::new(&query_result, &pinned_config, theme, config);
    app.update_search_filter();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new();
    let mut tui = Tui::new(terminal, events);

    tui.init()?;
    let result = run(&mut app, &mut tui);

    // Exit the user interface.
    tui.exit()?;
    result?;
    Ok(())
}

/// The main application execution loop.
///
/// This function coordinates the TUI rendering and event handling. It continues
/// looping until `app.running` is set to `false` (e.g., via a 'q' keypress).
///
/// # Arguments
/// * `app` - The mutable application state.
/// * `tui` - The terminal interface handle.
///
/// # Errors
/// Returns an error if drawing to the terminal fails or if an event handler
/// encounters a fatal error.
fn run(app: &mut App, tui: &mut Tui<CrosstermBackend<io::Stderr>>) -> AppResult<()> {
    while app.running {
        tui.draw(app)?;
        if let Event::Key(key_event) = tui.events.next()? {
            handle_key_events(key_event, app, tui.get_size()?.height)?;
        }
    }
    Ok(())
}

/// Resolves the initial arXiv query by merging command-line arguments and configuration settings.
///
/// # Priority Logic
/// 1. If an author is provided via `--author`, it is added to the query.
/// 2. If a category is provided via `--category`, it takes precedence.
/// 3. If no category arg is found, it falls back to the `query.category` defined in the config file.
///
/// # Errors
/// Returns an `AppResult` error if the arXiv API request fails or the XML response cannot be parsed.
fn resolve_query(args: &Args, config: &config::Config) -> AppResult<ArxivQueryResult> {
    let mut queries = Vec::new();

    // Handle Author
    if let Some(author) = &args.author {
        queries.push(SearchQuery::Author(author.to_string()));
    }

    // Handle Category: CLI arg takes priority, then Config fallback
    let category = args
        .category
        .clone()
        .unwrap_or_else(|| config.query.category.clone());

    queries.push(SearchQuery::Category(category));

    // Fetch XML
    let xml_body = query_arxiv(
        Some(&queries),
        Some(DEFAULT_START_INDEX),
        Some(DEFAULT_MAX_RESULTS),
        Some(DEFAULT_SORT_BY),
        Some(DEFAULT_SORT_ORDER),
    )
    .map_err(|e| format!("Failed to query arXiv: {e}"))?;

    // Parse the Result
    let result =
        ArxivQueryResult::from_xml_content(&xml_body).map_err(|e| format!("Parsing Error: {e}"))?;

    Ok(result)
}

/// Sets up a custom panic hook to ensure the terminal is restored to a usable state if the app crashes.
///
/// This function intercepts a `panic!`, disables terminal raw mode, leaves the alternate screen,
/// and shows the cursor before allowing the standard panic message to print.
/// Without this, a crash would leave the user's terminal session "broken" (no cursor, no echo).
fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // 1. Force the terminal to reset even during a crash
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stderr(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );

        // 2. Call the original hook to print the actual panic message
        original_hook(panic_info);
    }));
}
