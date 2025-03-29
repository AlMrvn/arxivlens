# arXivLens: A Terminal User Interface arXiv Explorer
ArxivLens is a Rust-based terminal user interface (TUI) application that helps you browse and explore new abstracts on the arXiv repository. The name was suggested by the AI assistant, Gemini!

## Motivation
This project arose from a desire to create a convenient way to explore the latest arXiv entries in specific categories (like "quant-ph"). The goal was to replicate the experience of browsing submitted manuscripts on the arXiv website, allowing you to scan through abstracts and search for keywords or familiar authors. Additionally, it served as a platform for myself to experiment and learn with the Rust programming language.

## Features

![TUI interface](screenshot.png)

- Browse new abstracts in your chosen category (default: "quant-ph")
- Search for specific authors in the arXiv database
- Highlight keywords and authors in abstracts and titles
- View detailed article information including authors, summary, and publication dates
- Customizable configuration for default category and highlighting preferences
- Fast and efficient terminal-based interface
- Support for all arXiv categories (e.g., quant-ph, cs.AI, math.AG)

## Installation
To install this as a CLI, you'll need [Rust installed](https://www.rust-lang.org/tools/install) then copy this repo and use cargo to compile the project into your path:
```bash
cargo install --path .
```
You will then be able to use the command `arxivlens` from any place in your system.

## Usage

### Command Line Options
```bash
arxivlens [OPTIONS]

Options:
  -a, --author <AUTHOR>      Name of the author to search for in arXiv entries
  -c, --category <CATEGORY>  ArXiv category to search (e.g., "quant-ph", "cs.AI")
  -h, --help                 Print help
  -V, --version              Print version
```

### Keyboard Shortcuts
- `↑` / `↓`: Navigate through article list
- `Enter`: View detailed article information
- `q`: Quit the application
- `Esc`: Return to article list from detail view

### Configuration
If `$XDG_CONFIG_HOME/arxivlens/config.toml` exists, it will be read and used. If `$XDG_CONFIG_HOME` is not set, `~/.cache/` will be used instead.

Example config file:
```toml
[query]
category = "quant-ph"

[highlight]
authors = ["Schrodinger", "Becquerel"]
keywords = ["quantum", "Error Correction"]
```

The configuration supports:
- Default category for arXiv queries
- List of authors to highlight in the article list
- List of keywords to highlight in abstracts and titles

## Examples

1. Browse quantum physics papers:
```bash
arxivlens
```

2. Search for a specific author in CS.AI category:
```bash
arxivlens -c "cs.AI" -a "Hinton"
```

3. Explore mathematics papers:
```bash
arxivlens -c "math.AG"
```

## Project Structure
The project is organized as follows:

```text
src/
├── arxiv_parsing.rs -> parsing of the XML returned by the arXiv API and search query
├── arxiv_query.rs   -> API for the arXiv API. Construction of the query url and 
├── app.rs           -> holds the state and application logic for the TUI
├── config.rs        -> handles the configuration for the query and the highlights
├── event.rs         -> handles the terminal events (key press, mouse click, resize, etc.)
├── handler.rs       -> handles the key press events and updates the application
├── lib.rs           -> module definitions
├── main.rs          -> entry-point
├── tui.rs           -> initializes/exits the terminal interface
└── ui.rs            -> renders the widgets / UI
```

## Contributing
Contributions are welcome! Please feel free to submit a Pull Request.

## License
This project is licensed under the MIT license.
