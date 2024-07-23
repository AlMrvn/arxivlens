use arxivlens::app::{App, AppResult};
use arxivlens::event::{Event, EventHandler};
use arxivlens::handler::handle_key_events;
use arxivlens::tui::Tui;
use clap::Parser;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

const DEFAULT_CATEGORY: &str = "quant-ph";

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
    let args = Args::parse();

    println!("Searching in {:?}!", args.category);
    println!("Searching in {:?}!", args.author);

    // Create an application.
    let mut app = App::new(args.category.as_deref(), args.author.as_deref());

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
