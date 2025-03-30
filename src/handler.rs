use crate::app::{App, AppResult};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            } else {
                app.toggle_config();
            }
        }
        // Counter handlers
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
        }
        // Movement a la Vim for 10 lines at a time
        // TODO: Make these movements half screen.
        KeyCode::Char('d') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                for _ in 0..10 {
                    app.select_next();
                }
            }
        }
        // TODO: Make this movement half screen
        KeyCode::Char('u') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                for _ in 0..10 {
                    app.select_previous();
                }
            }
        }
        KeyCode::Char('g') => {
            app.select_first();
        }
        KeyCode::Char('G') => {
            app.select_last();
        }
        KeyCode::Char('y') => {
            app.yank_id();
        }

        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}
