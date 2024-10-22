# arXivLens: A Terminal User Interface arXiv Explorer
ArxivLens is a Rust-based terminal user interface (TUI) application that helps you browse and explore new abstracts on the arXiv repository. The name was suggested by the AI assistant, Gemini!

## Motivation
This project arose from a desire to create a convenient way to explore the latest arXiv entries in specific categories (like "quant-ph"). The goal was to replicate the experience of browsing submitted manuscripts on the arXiv website, allowing you to scan through abstracts and search for keywords or familiar authors. Additionally, it served as a platform for myself to experiment and learn with the Rust programming language.

## Features

![TUI interface](screenshot.png)

- Browse new abstracts in your chosen category (default: "quant-ph").
- Highlight specific keywords within summaries (using -s flag).
- Search for authors you know of in the author list through arXiv query API (using -a flag).

## Configuration
If '$XDG_CONFIG_HOME/arxivlens/config.toml' exists, it will be read and used. If '$XDG_CONFIG_HOME' is not set, '~/.cache/' will be used instead.
Example of config file:

```toml
[query]
category = "quant-ph"

[highlight]
authors = ["Schrodinger", "Becquerel"]
keywords = ["quantum", "Error Correction"]

```

Here is the helper:

```text
Terminal User Interface to explore arXiv

Usage: arxivlens [OPTIONS]

Options:
  -a, --author <AUTHOR>                        Name of the author to look
  -c, --category <CATEGORY>                    Category to look [default: quant-ph]
  -h, --help                                   Print help
  -V, --version                                Print version
```

## Installation
To install this as a CLI, you'll need [Rust installed](https://www.rust-lang.org/tools/install) then copy this repo and use cargo to compile the project inot your path:
```bash
cargo install --path .
```
You will then be able to use the command `arxivlens` from any place in your system

## License:

This project is licensed under the MIT license.

## Rust cargo organization
I used the simple template from ratatui. the project is organized as following:

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
