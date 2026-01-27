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
}

impl KeyBind {
    /// Create a new key binding
    pub fn new(key: KeyCode, modifiers: KeyModifiers, action: Action) -> Self {
        Self {
            key,
            modifiers,
            action,
        }
    }
}

/// Default key mappings for the application
pub const KEY_MAP: &[KeyBind] = &[
    KeyBind {
        key: KeyCode::Char('q'),
        modifiers: KeyModifiers::empty(),
        action: Action::Quit,
    },
    KeyBind {
        key: KeyCode::Esc,
        modifiers: KeyModifiers::empty(),
        action: Action::Quit,
    },
    KeyBind {
        key: KeyCode::Char('j'),
        modifiers: KeyModifiers::empty(),
        action: Action::MoveDown,
    },
    KeyBind {
        key: KeyCode::Down,
        modifiers: KeyModifiers::empty(),
        action: Action::MoveDown,
    },
    KeyBind {
        key: KeyCode::Char('k'),
        modifiers: KeyModifiers::empty(),
        action: Action::MoveUp,
    },
    KeyBind {
        key: KeyCode::Up,
        modifiers: KeyModifiers::empty(),
        action: Action::MoveUp,
    },
    KeyBind {
        key: KeyCode::Char('g'),
        modifiers: KeyModifiers::empty(),
        action: Action::GoToTop,
    },
    KeyBind {
        key: KeyCode::Char('G'),
        modifiers: KeyModifiers::SHIFT,
        action: Action::GoToBottom,
    },
    KeyBind {
        key: KeyCode::Char('c'),
        modifiers: KeyModifiers::empty(),
        action: Action::ToggleConfig,
    },
    KeyBind {
        key: KeyCode::Char('?'),
        modifiers: KeyModifiers::empty(),
        action: Action::ShowHelp,
    },
    KeyBind {
        key: KeyCode::Char('y'),
        modifiers: KeyModifiers::empty(),
        action: Action::YankId,
    },
    KeyBind {
        key: KeyCode::Char('d'),
        modifiers: KeyModifiers::CONTROL,
        action: Action::PageDown,
    },
    KeyBind {
        key: KeyCode::Char('u'),
        modifiers: KeyModifiers::CONTROL,
        action: Action::PageUp,
    },
    KeyBind {
        key: KeyCode::Char('c'),
        modifiers: KeyModifiers::CONTROL,
        action: Action::Quit,
    },
    KeyBind {
        key: KeyCode::Char('C'),
        modifiers: KeyModifiers::CONTROL,
        action: Action::Quit,
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

    #[test]
    fn test_key_bind_creation() {
        let kb = KeyBind::new(KeyCode::Char('q'), KeyModifiers::empty(), Action::Quit);
        assert_eq!(kb.key, KeyCode::Char('q'));
        assert_eq!(kb.modifiers, KeyModifiers::empty());
        assert_eq!(kb.action, Action::Quit);
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
            Some(&Action::Quit)
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
        assert_eq!(key_map.len(), 15);
    }

    #[test]
    fn test_action_equality() {
        assert_eq!(Action::Quit, Action::Quit);
        assert_ne!(Action::Quit, Action::MoveUp);
    }
}
