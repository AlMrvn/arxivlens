# arXivLens: A Terminal User Interface arXiv Explorer
ArxivLens is a Rust-based terminal user interface (TUI) application that helps you browse and explore new abstracts on the arXiv repository. The name was suggested by the AI assistant, Gemini!

## Motivation
This project arose from a desire to create a convenient way to explore the latest arXiv entries in specific categories (like "quant-ph"). The goal was to replicate the experience of browsing submitted manuscripts on the arXiv website, allowing you to scan through abstracts and search for keywords or familiar authors. Additionally, it served as a platform for myself to experiment and learn with the Rust programming language and LLM coding more recently.

## Features

![TUI interface](screenshot.png)

## Installation

### From crates.io (Stable Release)
```bash
cargo install arxivlens
```

### From GitHub (Development Version)
```bash
# Clone the repository
git clone https://github.com/yourusername/arxivlens.git
cd arxivlens

# Install the development version
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
| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Ctrl+d` / `PgDn` | Page down |
| `Ctrl+u` / `PgUp` | Page up |
| `g` | Go to top |
| `G` | Go to bottom |
| `/` | Open Search |
| `y` | Yank (copy) Article ID to clipboard |
| `c` | Toggle Config popup |
| `?` | Show Help |
| `Esc` | Close popup or Exit |
| `q` | Quit |

### Configuration
If `$XDG_CONFIG_HOME/arxivlens/config.toml` exists, it will be read and used. If `$XDG_CONFIG_HOME` is not set, `~/.cache/` will be used instead.

Example config file:
```toml
[query]
category = "quant-ph"

[ui]
theme_name = "dark"

[pinned]
authors = ["Schrodinger", "Becquerel"]
categories = ["quant-ph", "cs.AI"]
```

The configuration supports:
- Default category for arXiv queries
- List of authors to highlight in the article list

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
## UI Testing & Golden Files

This project uses Golden File testing (snapshot testing) for UI components to prevent layout regressions. Golden files capture the expected terminal output for each component and are compared against actual output during tests.

### Running Tests

Run all tests (including golden file comparisons):
```bash
cargo test
```

Run only the UI integration tests:
```bash
cargo test --test ui_golden_tests
```

### Updating Golden Files

When you intentionally change the UI design and need to update the expected output:
```bash
UPDATE_GOLDEN=1 cargo test --test ui_golden_tests
```

Golden files are stored in `tests/golden/` and should be committed to version control as they represent the expected UI behavior.

## Contributing
Contributions are welcome! Please feel free to submit a Pull Request.

## Development Setup
To ensure code quality and architectural boundaries, please run:
`git config core.hooksPath .githooks`

## License
This project is licensed under the MIT license.
