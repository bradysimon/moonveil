use std::fmt::{self, Display};

use iced_core::keyboard::{self, key};

/// A typed keyboard shortcut displayed beside an action item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortcut {
    pub modifiers: keyboard::Modifiers,
    pub key: Key,
}

impl Shortcut {
    /// Creates a new keyboard shortcut.
    pub const fn new(modifiers: keyboard::Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }

    /// Creates a platform command-key shortcut (Cmd on macOS, Ctrl elsewhere).
    pub const fn command(key: Key) -> Self {
        Self::new(keyboard::Modifiers::COMMAND, key)
    }

    /// Creates a control-key shortcut.
    pub const fn control(key: Key) -> Self {
        Self::new(keyboard::Modifiers::CTRL, key)
    }

    /// Creates a shift + platform command-key shortcut.
    pub const fn shift_command(key: Key) -> Self {
        Self::new(
            keyboard::Modifiers::COMMAND.union(keyboard::Modifiers::SHIFT),
            key,
        )
    }

    /// Returns `true` if the pressed key and modifiers trigger this shortcut.
    pub fn matches(&self, key: &keyboard::Key, modifiers: keyboard::Modifiers) -> bool {
        modifiers == self.modifiers && self.key.matches(key)
    }
}

impl Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let modifiers = self.modifiers;
        let labels = if cfg!(target_os = "macos") {
            [
                (modifiers.control(), "Ctrl"),
                (modifiers.alt(), "Opt"),
                (modifiers.shift(), "Shift"),
                (modifiers.logo(), "Cmd"),
            ]
        } else {
            [
                (modifiers.control(), "Ctrl"),
                (modifiers.alt(), "Alt"),
                (modifiers.shift(), "Shift"),
                (modifiers.logo(), "Super"),
            ]
        };

        for (_, label) in labels.iter().filter(|(pressed, _)| *pressed) {
            write!(f, "{label}+")?;
        }

        match self.key {
            Key::Character(character) => write!(f, "{}", character.to_ascii_uppercase()),
            Key::Named(named) => f.write_str(named_key_label(named)),
        }
    }
}

fn named_key_label(named: key::Named) -> &'static str {
    match named {
        key::Named::Enter => "Enter",
        key::Named::Tab => "Tab",
        key::Named::Space => "Space",
        key::Named::Escape => "Esc",
        key::Named::Backspace => "Backspace",
        key::Named::Delete => "Delete",
        key::Named::ArrowUp => "Up",
        key::Named::ArrowDown => "Down",
        key::Named::ArrowLeft => "Left",
        key::Named::ArrowRight => "Right",
        _ => "Key",
    }
}

/// The key portion of a [`Shortcut`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// A character key.
    Character(char),
    /// A named keyboard key.
    Named(key::Named),
}

impl Key {
    /// Returns `true` if the pressed key matches this key, ignoring ASCII case.
    pub fn matches(self, key: &keyboard::Key) -> bool {
        match (self, key.as_ref()) {
            (Self::Character(expected), keyboard::Key::Character(actual)) => actual
                .chars()
                .next()
                .is_some_and(|actual| actual.eq_ignore_ascii_case(&expected)),
            (Self::Named(expected), keyboard::Key::Named(actual)) => expected == actual,
            _ => false,
        }
    }
}
