use std::borrow::Cow;

use iced_core::keyboard;

use super::shortcut::Shortcut;

/// A context menu item.
#[derive(Debug, Clone)]
pub enum Item<'a, Message> {
    /// A clickable action item.
    Action {
        title: Cow<'a, str>,
        on_press: Option<Message>,
        shortcut: Option<Shortcut>,
    },
    /// A nested submenu item.
    Submenu {
        title: Cow<'a, str>,
        items: Vec<Item<'a, Message>>,
        enabled: bool,
    },
    /// A visual separator between item groups.
    Separator,
}

impl<'a, Message> Item<'a, Message> {
    /// Adds a keyboard shortcut to an action item.
    #[must_use]
    pub fn shortcut(mut self, shortcut: Shortcut) -> Self {
        if let Self::Action {
            shortcut: item_shortcut,
            ..
        } = &mut self
        {
            *item_shortcut = Some(shortcut);
        }

        self
    }

    /// Enables or disables an action or submenu item.
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        match &mut self {
            Self::Action { on_press, .. } if disabled => *on_press = None,
            Self::Submenu { enabled, .. } => *enabled = !disabled,
            _ => {}
        }

        self
    }

    pub(super) fn title(&self) -> Option<&str> {
        match self {
            Self::Action { title, .. } | Self::Submenu { title, .. } => Some(title.as_ref()),
            Self::Separator => None,
        }
    }

    pub(super) fn is_enabled(&self) -> bool {
        match self {
            Self::Action { on_press, .. } => on_press.is_some(),
            Self::Submenu { enabled, items, .. } => *enabled && has_navigable_item(items),
            Self::Separator => false,
        }
    }

    pub(super) fn submenu_items(&self) -> Option<&[Item<'a, Message>]> {
        match self {
            Self::Submenu { items, enabled, .. } if *enabled => Some(items),
            _ => None,
        }
    }

    pub(super) fn message(&self) -> Option<&Message> {
        match self {
            Self::Action { on_press, .. } => on_press.as_ref(),
            _ => None,
        }
    }

    pub(super) fn shortcut_value(&self) -> Option<Shortcut> {
        match self {
            Self::Action { shortcut, .. } => *shortcut,
            _ => None,
        }
    }
}

pub(super) fn first_navigable_path<Message>(items: &[Item<'_, Message>]) -> Option<Vec<usize>> {
    first_navigable_index(items).map(|index| vec![index])
}

pub(super) fn first_navigable_index<Message>(items: &[Item<'_, Message>]) -> Option<usize> {
    items.iter().position(Item::is_enabled)
}

pub(super) fn last_navigable_index<Message>(items: &[Item<'_, Message>]) -> Option<usize> {
    items.iter().rposition(Item::is_enabled)
}

pub(super) fn has_navigable_item<Message>(items: &[Item<'_, Message>]) -> bool {
    first_navigable_index(items).is_some()
}

pub(super) fn next_navigable_index<Message>(
    items: &[Item<'_, Message>],
    current_index: usize,
    delta: isize,
) -> Option<usize> {
    let len = items.len();
    if len == 0 {
        return None;
    }

    let mut index = current_index;
    for _ in 0..len {
        index = if delta > 0 {
            (index + 1) % len
        } else if index == 0 {
            len - 1
        } else {
            index - 1
        };

        if items[index].is_enabled() {
            return Some(index);
        }
    }

    None
}

/// Recursively searches for an enabled action with a matching shortcut.
pub(super) fn shortcut_message<'a, Message>(
    items: &'a [Item<'a, Message>],
    key: &keyboard::Key,
    modifiers: keyboard::Modifiers,
) -> Option<&'a Message> {
    for item in items {
        match item {
            Item::Action {
                on_press: Some(message),
                shortcut: Some(shortcut),
                ..
            } if shortcut.matches(key, modifiers) => return Some(message),
            Item::Submenu { items, enabled, .. } if *enabled => {
                if let Some(message) = shortcut_message(items, key, modifiers) {
                    return Some(message);
                }
            }
            _ => {}
        }
    }

    None
}

/// Returns the items and active index of the deepest level in `active_path`.
pub(super) fn active_level<'a, Message>(
    items: &'a [Item<'a, Message>],
    active_path: &[usize],
) -> Option<(&'a [Item<'a, Message>], usize)> {
    let (&current_index, parents) = active_path.split_last()?;
    let mut current_items = items;

    for index in parents {
        current_items = current_items.get(*index)?.submenu_items()?;
    }

    Some((current_items, current_index))
}

/// Moves the deepest active index by `delta`, skipping disabled items.
pub(super) fn move_active<Message>(
    items: &[Item<'_, Message>],
    active_path: &mut Vec<usize>,
    delta: isize,
) {
    if active_path.is_empty() {
        let next_index = if delta >= 0 {
            first_navigable_index(items)
        } else {
            last_navigable_index(items)
        };

        if let Some(next_index) = next_index {
            active_path.push(next_index);
        }

        return;
    }

    let Some((current_items, current_index)) = active_level(items, active_path) else {
        return;
    };

    let Some(next_index) = next_navigable_index(current_items, current_index, delta) else {
        return;
    };

    if let Some(last) = active_path.last_mut() {
        *last = next_index;
    }
}

pub(super) fn active_message<'a, Message>(
    items: &'a [Item<'a, Message>],
    path: &[usize],
) -> Option<&'a Message> {
    item_at_path(items, path).and_then(Item::message)
}

pub(super) fn item_at_path<'a, Message>(
    items: &'a [Item<'a, Message>],
    path: &[usize],
) -> Option<&'a Item<'a, Message>> {
    let mut current_items = items;
    let mut current_item = None;

    for (depth, index) in path.iter().copied().enumerate() {
        let item = current_items.get(index)?;
        current_item = Some(item);

        if depth + 1 < path.len() {
            current_items = item.submenu_items()?;
        }
    }

    current_item
}
