use crate::app::Context;
use crate::app::SearchAction;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use std::collections::HashMap;

/// Represents the different actions that can be performed in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Quit the application
    Quit,
    /// Move selection up
    MoveUp,
    /// Move selection down
    MoveDown,
    /// Jump up multiple lines (page up)
    PageUp,
    /// Jump down multiple lines (page down)
    PageDown,
    /// Go to the top/first item
    GoToTop,
    /// Go to the bottom/last item
    GoToBottom,
    /// Toggle the configuration popup
    ToggleConfig,
    /// Show help information
    ShowHelp,
    /// Yank (copy) the selected article ID
    YankId,
    /// Close popup or quit if no popup is open
    ClosePopup,
    /// Enter search mode
    Search,
    /// Typing in the search
    SearchInput(SearchAction),
    CycleFocus,
}

impl Action {
    /// Check if this action is valid in the given context
    pub fn is_valid_in(&self, context: &crate::app::Context) -> bool {
        match self {
            // These actions are always valid regardless of context
            Action::Quit | Action::ShowHelp | Action::ToggleConfig => true,
            // Navigation and yanking actions are only valid in ArticleList context
            Action::MoveUp
            | Action::MoveDown
            | Action::PageUp
            | Action::PageDown
            | Action::GoToTop
            | Action::GoToBottom
            | Action::YankId => matches!(context, Context::ArticleList | Context::Pinned),
            // ClosePopup is always valid - behavior depends on context
            Action::ClosePopup => true,
            // Search is always valid
            Action::Search => true,
            Action::CycleFocus => true,
            Action::SearchInput(_) => *context == Context::Search,
        }
    }

    /// Get a short description of the action
    pub fn description(&self) -> &str {
        match self {
            Action::Quit => "Quit",
            Action::MoveUp => "Up",
            Action::MoveDown => "Down",
            Action::PageUp => "Page Up",
            Action::PageDown => "Page Down",
            Action::GoToTop => "Go to Top",
            Action::GoToBottom => "Go to Bottom",
            Action::ToggleConfig => "Config",
            Action::ShowHelp => "Help",
            Action::YankId => "Yank",
            Action::ClosePopup => "Close/Quit",
            Action::Search => "Search",
            Action::CycleFocus => "Cycle Focus",
            Action::SearchInput(_) => "Type to search",
        }
    }
}

/// Represents a key binding configuration
#[derive(Debug, Clone)]
pub struct KeyBind {
    /// The key code that triggers this action
    pub key: KeyCode,
    /// The modifiers required for this key binding
    pub modifiers: KeyModifiers,
    /// The action to perform when this key is pressed
    pub action: Action,
    /// Whether this is a primary key binding (shown in footer)
    pub is_primary: bool,
}

impl KeyBind {
    /// Create a new key binding
    pub fn new(key: KeyCode, modifiers: KeyModifiers, action: Action, is_primary: bool) -> Self {
        Self {
            key,
            modifiers,
            action,
            is_primary,
        }
    }
}

/// Default key mappings for the application
pub const KEY_MAP: &[KeyBind] = &[
    KeyBind {
        key: KeyCode::Char('q'),
        modifiers: KeyModifiers::empty(),
        action: Action::Quit,
        is_primary: true,
    },
    KeyBind {
        key: KeyCode::Esc,
        modifiers: KeyModifiers::empty(),
        action: Action::ClosePopup,
        is_primary: false,
    },
    KeyBind {
        key: KeyCode::Char('j'),
        modifiers: KeyModifiers::empty(),
        action: Action::MoveDown,
        is_primary: true,
    },
    KeyBind {
        key: KeyCode::Down,
        modifiers: KeyModifiers::empty(),
        action: Action::MoveDown,
        is_primary: false,
    },
    KeyBind {
        key: KeyCode::Char('k'),
        modifiers: KeyModifiers::empty(),
        action: Action::MoveUp,
        is_primary: true,
    },
    KeyBind {
        key: KeyCode::Up,
        modifiers: KeyModifiers::empty(),
        action: Action::MoveUp,
        is_primary: false,
    },
    KeyBind {
        key: KeyCode::Char('g'),
        modifiers: KeyModifiers::empty(),
        action: Action::GoToTop,
        is_primary: false,
    },
    KeyBind {
        key: KeyCode::Char('G'),
        modifiers: KeyModifiers::SHIFT,
        action: Action::GoToBottom,
        is_primary: false,
    },
    KeyBind {
        key: KeyCode::Char('c'),
        modifiers: KeyModifiers::empty(),
        action: Action::ToggleConfig,
        is_primary: true,
    },
    KeyBind {
        key: KeyCode::Char('?'),
        modifiers: KeyModifiers::empty(),
        action: Action::ShowHelp,
        is_primary: true,
    },
    KeyBind {
        key: KeyCode::Char('y'),
        modifiers: KeyModifiers::empty(),
        action: Action::YankId,
        is_primary: true,
    },
    KeyBind {
        key: KeyCode::Char('d'),
        modifiers: KeyModifiers::CONTROL,
        action: Action::PageDown,
        is_primary: false,
    },
    KeyBind {
        key: KeyCode::Char('u'),
        modifiers: KeyModifiers::CONTROL,
        action: Action::PageUp,
        is_primary: false,
    },
    KeyBind {
        key: KeyCode::Char('c'),
        modifiers: KeyModifiers::CONTROL,
        action: Action::Quit,
        is_primary: false,
    },
    KeyBind {
        key: KeyCode::Char('C'),
        modifiers: KeyModifiers::CONTROL,
        action: Action::Quit,
        is_primary: false,
    },
    KeyBind {
        key: KeyCode::Char('/'),
        modifiers: KeyModifiers::empty(),
        action: Action::Search,
        is_primary: true,
    },
    KeyBind {
        key: KeyCode::Tab,
        modifiers: KeyModifiers::empty(),
        action: Action::CycleFocus,
        is_primary: true,
    },
];

/// Helper function to create a HashMap from the key mappings for quick lookup
pub fn create_key_map() -> HashMap<(KeyCode, KeyModifiers), Action> {
    KEY_MAP
        .iter()
        .map(|kb| ((kb.key, kb.modifiers), kb.action))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::app::{tests::create_test_app, Context};
    use crate::handler::handle_key_events;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_key_bind_creation() {
        let kb = KeyBind::new(
            KeyCode::Char('q'),
            KeyModifiers::empty(),
            Action::Quit,
            true,
        );
        assert_eq!(kb.key, KeyCode::Char('q'));
        assert_eq!(kb.modifiers, KeyModifiers::empty());
        assert_eq!(kb.action, Action::Quit);
        assert!(kb.is_primary);
    }

    #[test]
    fn test_key_map_contains_expected_keys() {
        let key_map = create_key_map();
        assert_eq!(
            key_map.get(&(KeyCode::Char('q'), KeyModifiers::empty())),
            Some(&Action::Quit)
        );
        assert_eq!(
            key_map.get(&(KeyCode::Esc, KeyModifiers::empty())),
            Some(&Action::ClosePopup)
        );
        assert_eq!(
            key_map.get(&(KeyCode::Char('j'), KeyModifiers::empty())),
            Some(&Action::MoveDown)
        );
        assert_eq!(
            key_map.get(&(KeyCode::Down, KeyModifiers::empty())),
            Some(&Action::MoveDown)
        );
        assert_eq!(
            key_map.get(&(KeyCode::Char('k'), KeyModifiers::empty())),
            Some(&Action::MoveUp)
        );
        assert_eq!(
            key_map.get(&(KeyCode::Up, KeyModifiers::empty())),
            Some(&Action::MoveUp)
        );
        assert_eq!(
            key_map.get(&(KeyCode::Char('g'), KeyModifiers::empty())),
            Some(&Action::GoToTop)
        );
        assert_eq!(
            key_map.get(&(KeyCode::Char('G'), KeyModifiers::SHIFT)),
            Some(&Action::GoToBottom)
        );
        assert_eq!(
            key_map.get(&(KeyCode::Char('c'), KeyModifiers::empty())),
            Some(&Action::ToggleConfig)
        );
        assert_eq!(
            key_map.get(&(KeyCode::Char('?'), KeyModifiers::empty())),
            Some(&Action::ShowHelp)
        );
        assert_eq!(
            key_map.get(&(KeyCode::Char('y'), KeyModifiers::empty())),
            Some(&Action::YankId)
        );
        assert_eq!(
            key_map.get(&(KeyCode::Char('d'), KeyModifiers::CONTROL)),
            Some(&Action::PageDown)
        );
        assert_eq!(
            key_map.get(&(KeyCode::Char('u'), KeyModifiers::CONTROL)),
            Some(&Action::PageUp)
        );
    }

    #[test]
    fn test_key_map_size() {
        let key_map = create_key_map();
        assert_eq!(key_map.len(), 17);
    }

    #[test]
    fn test_key_collision_prevention() {
        let key_map = create_key_map();

        // Test that 'c' maps to ToggleConfig
        assert_eq!(
            key_map.get(&(KeyCode::Char('c'), KeyModifiers::empty())),
            Some(&Action::ToggleConfig)
        );

        // Test that 'Ctrl+c' maps to Quit
        assert_eq!(
            key_map.get(&(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(&Action::Quit)
        );

        // Ensure they are different actions
        assert_ne!(Action::ToggleConfig, Action::Quit);
    }

    #[test]
    fn test_action_equality() {
        assert_eq!(Action::Quit, Action::Quit);
        assert_ne!(Action::Quit, Action::MoveUp);
    }

    #[test]
    fn test_esc_closes_popup_not_app() {
        let mut app = create_test_app();

        // 1. Test closing Help
        app.set_context(Context::Help);
        assert_eq!(app.current_context, Context::Help);

        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        handle_key_events(event, &mut app, 20).unwrap();

        assert_eq!(app.current_context, Context::ArticleList);
        assert!(app.running);

        // 2. Test closing Config
        app.set_context(Context::Config);
        assert_eq!(app.current_context, Context::Config);

        handle_key_events(event, &mut app, 20).unwrap();

        assert_eq!(app.current_context, Context::ArticleList);
        assert!(app.running);
    }
}
