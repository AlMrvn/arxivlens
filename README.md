This project provides a text-based user interface (TUI) application written in Rust for browsing and reading new abstracts on arXiv.

## License:

This project is licensed under the MIT license.

## Rust cargo orgamization
I used the simple template from ratatui. the project is organized as following:

```text
src/
├── app.rs     -> holds the state and application logic
├── event.rs   -> handles the terminal events (key press, mouse click, resize, etc.)
├── handler.rs -> handles the key press events and updates the application
├── lib.rs     -> module definitions
├── main.rs    -> entry-point
├── tui.rs     -> initializes/exits the terminal interface
└── ui.rs      -> renders the widgets / UI
```
