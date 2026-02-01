use crate::app::actions::{Action, KEY_MAP};
use crate::app::{App, AppResult};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Maps a key event to an action based on the KEY_MAP
fn map_key_to_action(key_event: KeyEvent) -> Option<Action> {
    KEY_MAP
        .iter()
        .find(|keybind| keybind.key == key_event.code && keybind.modifiers == key_event.modifiers)
        .map(|keybind| keybind.action)
}

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    terminal_height: u16,
) -> AppResult<()> {
    // 1. Modal Logic: If we are typing a search, handle characters directly
    if matches!(app.current_context, crate::app::Context::Search) {
        match key_event.code {
            KeyCode::Char(c) => {
                app.handle_search_char(c);
                return Ok(());
            }
            KeyCode::Backspace => {
                app.handle_search_backspace();
                return Ok(());
            }
            KeyCode::Esc | KeyCode::Enter => {
                app.set_context(crate::app::Context::ArticleList);
                return Ok(());
            }
            _ => {} // Ignore other keys while searching
        }
    }

    // 2. Global Navigation: LazyGit-style shortcuts
    // This intercepts digits 1-9 to switch contexts globally
    if let KeyCode::Char(c) = key_event.code {
        if c.is_ascii_digit() && c != '0' {
            let digit = c.to_digit(10).unwrap() as usize;
            if app.navigate_to_shortcut(digit) {
                return Ok(()); // Shortcut handled, stop processing
            }
        }
    }

    // 2. Action Logic: For everything else, use the KEY_MAP
    if let Some(action) = map_key_to_action(key_event) {
        if action.is_valid_in(&app.current_context) {
            app.perform_action(action, terminal_height);
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
        let last_index = app.get_visible_count().saturating_sub(1);

        let event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
        handle_key_events(event, &mut app, 20).unwrap();

        assert_eq!(app.article_list_state.selected(), Some(last_index));
    }

    #[test]
    fn test_handle_g_to_top() {
        let mut app = create_test_app();

        // First move to the bottom
        app.select_last();
        let last_index = app.get_visible_count().saturating_sub(1);
        assert_eq!(app.selected_index(), Some(last_index));

        // Create a KeyEvent for 'g' (lowercase g)
        let key_event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty());

        handle_key_events(key_event, &mut app, 20).unwrap();

        assert_eq!(app.selected_index(), Some(0));
    }

    #[test]
    fn test_handle_quit_key() {
        let mut app = create_test_app();
        assert!(app.running);

        // Create a KeyEvent for 'q'
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());

        // Handle the key event
        handle_key_events(key_event, &mut app, 20).unwrap();

        // Assert that the app is no longer running
        assert!(!app.running);
    }

    #[test]
    fn test_handle_j_and_k() {
        let mut app = create_test_app();
        let max_index = app.get_visible_count().saturating_sub(1);

        // --- Test 'j' (Down) ---
        app.article_list_state.select(Some(0)); // Start at top
        let initial = app.article_list_state.selected().unwrap_or(0);

        handle_key_events(
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
            &mut app,
            20,
        )
        .unwrap();

        let expected_down = (initial + 1).min(max_index);
        assert_eq!(
            app.article_list_state.selected(),
            Some(expected_down),
            "Failed on 'j' move"
        );

        // --- Test 'k' (Up) ---
        let current = app.article_list_state.selected().unwrap_or(0);

        handle_key_events(
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
            &mut app,
            20,
        )
        .unwrap();

        let expected_up = current.saturating_sub(1);
        assert_eq!(
            app.article_list_state.selected(),
            Some(expected_up),
            "Failed on 'k' move"
        );
    }

    #[test]
    fn test_handle_ctrl_d_page_down() {
        let mut app = create_test_app();
        let max_index = app.get_visible_count().saturating_sub(1);

        // Simulate Ctrl+D with terminal height
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        handle_key_events(event, &mut app, 20).unwrap(); // 20 height = 10 step

        // We expect it to be clamped to the very bottom
        assert_eq!(app.article_list_state.selected(), Some(max_index));
    }

    #[test]
    fn test_handle_ctrl_u_page_up() {
        let mut app = create_test_app();
        // 1. Move to the bottom first (index 4)
        app.article_list_state.select(Some(4));

        // 2. Simulate Ctrl+U (Page Up - usually a step of 10)
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        handle_key_events(event, &mut app, 20).unwrap(); // 20 height = 10 step

        // 3. Assert it clamped to 0
        assert_eq!(app.article_list_state.selected(), Some(0));
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

            handle_key_events(event, &mut app, 20).unwrap();

            assert!(!app.running, "Key {:?} failed to quit the app", event.code);
        }
    }

    #[test]
    fn test_navigation_disabled_in_config_context() {
        let mut app = create_test_app();

        // Set the context to Config
        app.set_context(crate::app::Context::Config);

        // Capture the initial selected index
        let initial_index = app.selected_index();

        // Send a MoveDown key event
        let key_event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());
        handle_key_events(key_event, &mut app, 20).unwrap();

        // Assert that the selected index has NOT changed
        assert_eq!(
            app.selected_index(),
            initial_index,
            "Navigation should be disabled in Config context"
        );
    }

    #[test]
    fn test_help_context_lock() {
        let mut app = create_test_app();

        // Set the context to Help
        app.set_context(crate::app::Context::Help);

        // Capture the initial selected index
        let initial_index = app.selected_index();

        // Send a MoveDown key event
        let key_event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());
        handle_key_events(key_event, &mut app, 20).unwrap();

        // Assert that the selected index has NOT changed
        assert_eq!(
            app.selected_index(),
            initial_index,
            "Navigation should be disabled in Help context"
        );
    }

    #[test]
    fn test_smart_esc_logic() {
        let mut app = create_test_app();

        // Test Esc in ArticleList context (should quit)
        app.set_context(crate::app::Context::ArticleList);
        assert!(app.running);

        let esc_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        handle_key_events(esc_event, &mut app, 20).unwrap();

        assert!(!app.running, "Esc should quit when in ArticleList context");

        // Test Esc in Config context (should close popup)
        let mut app = create_test_app();
        app.set_context(crate::app::Context::Config);

        let esc_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        handle_key_events(esc_event, &mut app, 20).unwrap();

        assert_eq!(app.current_context, crate::app::Context::ArticleList);
        assert!(
            app.running,
            "App should still be running after closing popup"
        );

        // Test Esc in Help context (should close help)
        let mut app = create_test_app();
        app.set_context(crate::app::Context::Help);

        let esc_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        handle_key_events(esc_event, &mut app, 20).unwrap();

        assert_eq!(app.current_context, crate::app::Context::ArticleList);
        assert!(
            app.running,
            "App should still be running after closing help"
        );
    }

    #[test]
    fn test_dynamic_scrolling_math() {
        let app = create_test_app();

        // Test with very small terminal height
        let small_step = app.calculate_half_page_step(1);
        assert_eq!(small_step, 1, "Should have minimum step of 1 for height 1");

        let small_step = app.calculate_half_page_step(2);
        assert_eq!(small_step, 1, "Should have step of 1 for height 2");

        let normal_step = app.calculate_half_page_step(20);
        assert_eq!(normal_step, 10, "Should have step of 10 for height 20");

        let large_step = app.calculate_half_page_step(100);
        assert_eq!(large_step, 50, "Should have step of 50 for height 100");
    }
}
