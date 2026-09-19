use std::borrow::Cow;

use iced_core::{Pixels, text};

/// A dropdown menu item.
#[derive(Debug, Clone)]
pub enum Item<'a, Message, Font = iced_core::Font> {
    /// A clickable action item.
    Action {
        title: Cow<'a, str>,
        on_press: Option<Message>,
        leading: Option<Leading<Font>>,
    },
    /// A non-interactive section header.
    Section { title: Cow<'a, str> },
    /// A visual separator between item groups.
    Separator,
}

impl<Message, Font> Item<'_, Message, Font> {
    /// Sets the leading font glyph of an action item.
    #[must_use]
    pub fn leading(mut self, leading: impl Into<Leading<Font>>) -> Self {
        if let Self::Action {
            leading: item_leading,
            ..
        } = &mut self
        {
            *item_leading = Some(leading.into());
        }

        self
    }

    /// Enables or disables an action item.
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        if let Self::Action { on_press, .. } = &mut self
            && disabled
        {
            *on_press = None;
        }

        self
    }

    pub(super) fn is_enabled(&self) -> bool {
        matches!(
            self,
            Self::Action {
                on_press: Some(_),
                ..
            }
        )
    }

    pub(super) fn message(&self) -> Option<&Message> {
        match self {
            Self::Action { on_press, .. } => on_press.as_ref(),
            Self::Section { .. } | Self::Separator => None,
        }
    }

    pub(super) fn has_leading(&self) -> bool {
        matches!(
            self,
            Self::Action {
                leading: Some(_),
                ..
            }
        )
    }
}

/// A font glyph drawn at the leading edge of a dropdown action row.
#[derive(Debug, Clone)]
pub struct Leading<Font = iced_core::Font> {
    pub(super) font: Font,
    pub(super) code_point: char,
    pub(super) size: Option<Pixels>,
    pub(super) line_height: text::LineHeight,
    pub(super) shaping: text::Shaping,
}

impl<Font> Leading<Font> {
    /// Creates a font glyph leading icon.
    pub fn text(font: Font, code_point: char) -> Self {
        Self {
            font,
            code_point,
            size: None,
            line_height: text::LineHeight::default(),
            shaping: text::Shaping::Basic,
        }
    }

    /// Sets the size of the font glyph.
    #[must_use]
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the line height of the font glyph.
    #[must_use]
    pub fn line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the shaping strategy of the font glyph.
    #[must_use]
    pub fn shaping(mut self, shaping: text::Shaping) -> Self {
        self.shaping = shaping;
        self
    }
}

pub(super) fn first_navigable_index<Message, Font>(
    items: &[Item<'_, Message, Font>],
) -> Option<usize> {
    items.iter().position(Item::is_enabled)
}

pub(super) fn last_navigable_index<Message, Font>(
    items: &[Item<'_, Message, Font>],
) -> Option<usize> {
    items.iter().rposition(Item::is_enabled)
}

pub(super) fn next_navigable_index<Message, Font>(
    items: &[Item<'_, Message, Font>],
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
