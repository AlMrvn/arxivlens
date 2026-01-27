use crate::app::actions::{Action, KEY_MAP};
use crate::app::{App, AppResult};
use ratatui::crossterm::event::KeyEvent;

/// Maps a key event to an action based on the KEY_MAP
fn map_key_to_action(key_event: KeyEvent) -> Option<Action> {
    KEY_MAP
        .iter()
        .find(|keybind| keybind.key == key_event.code && keybind.modifiers == key_event.modifiers)
        .map(|keybind| keybind.action)
}

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if let Some(action) = map_key_to_action(key_event) {
        match action {
            Action::Quit => app.quit(),
            Action::MoveUp => app.select_previous(),
            Action::MoveDown => app.select_next(),
            Action::PageUp => app.scroll_up(10),
            Action::PageDown => app.scroll_down(10),
            Action::GoToTop => app.select_first(),
            Action::GoToBottom => app.select_last(),
            Action::ToggleConfig => app.toggle_config(),
            Action::ShowHelp => {
                // TODO: Implement show help functionality
            }
            Action::YankId => app.yank_id(),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::create_test_app;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_handle_g_to_bottom() {
        let mut app = create_test_app();
        let last_index = app.article_feed.len.saturating_sub(1);

        let event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
        handle_key_events(event, &mut app).unwrap();

        assert_eq!(app.article_feed.state.selected(), Some(last_index));
    }

    #[test]
    fn test_handle_g_to_top() {
        let mut app = create_test_app();

        // First move to the bottom
        app.select_last();
        let last_index = app.article_feed.len.saturating_sub(1);
        assert_eq!(app.selected_index(), Some(last_index));

        // Create a KeyEvent for 'g' (lowercase g)
        let key_event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty());

        handle_key_events(key_event, &mut app).unwrap();

        assert_eq!(app.selected_index(), Some(0));
    }

    #[test]
    fn test_handle_quit_key() {
        let mut app = create_test_app();
        assert!(app.running);

        // Create a KeyEvent for 'q'
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());

        // Handle the key event
        handle_key_events(key_event, &mut app).unwrap();

        // Assert that the app is no longer running
        assert!(!app.running);
    }

    #[test]
    fn test_handle_j_and_k() {
        let mut app = create_test_app();
        let max_index = app.article_feed.len.saturating_sub(1);

        // --- Test 'j' (Down) ---
        app.article_feed.state.select(Some(0)); // Start at top
        let initial = app.article_feed.state.selected().unwrap_or(0);

        handle_key_events(
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
            &mut app,
        )
        .unwrap();

        let expected_down = (initial + 1).min(max_index);
        assert_eq!(
            app.article_feed.state.selected(),
            Some(expected_down),
            "Failed on 'j' move"
        );

        // --- Test 'k' (Up) ---
        let current = app.article_feed.state.selected().unwrap_or(0);

        handle_key_events(
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
            &mut app,
        )
        .unwrap();

        let expected_up = current.saturating_sub(1);
        assert_eq!(
            app.article_feed.state.selected(),
            Some(expected_up),
            "Failed on 'k' move"
        );
    }

    #[test]
    fn test_handle_ctrl_d_page_down() {
        let mut app = create_test_app();
        let max_index = app.article_feed.len.saturating_sub(1);

        // Simulate Ctrl+D
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        handle_key_events(event, &mut app).unwrap();

        // We expect it to be clamped to the very bottom
        assert_eq!(app.article_feed.state.selected(), Some(max_index));
    }

    #[test]
    fn test_handle_ctrl_u_page_up() {
        let mut app = create_test_app();
        // 1. Move to the bottom first (index 4)
        app.article_feed.state.select(Some(4));

        // 2. Simulate Ctrl+U (Page Up - usually a step of 10)
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        handle_key_events(event, &mut app).unwrap();

        // 3. Assert it clamped to 0
        assert_eq!(app.article_feed.state.selected(), Some(0));
    }

    #[test]
    fn test_handle_quit_actions() {
        let quit_keys = vec![
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        ];

        for event in quit_keys {
            let mut app = create_test_app(); // Fresh app for each key
            assert!(app.running);

            handle_key_events(event, &mut app).unwrap();

            assert!(!app.running, "Key {:?} failed to quit the app", event.code);
        }
    }
}
