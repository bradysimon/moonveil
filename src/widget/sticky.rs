//! Keep content in view while its scrollable ancestor scrolls past it.

use crate::{Element, Theme};

pub type Sticky<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Sticky<'a, Message, Theme, Renderer>;

/// Creates a [`Sticky`] containing an element.
pub fn sticky<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Sticky<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    Sticky::new(content)
}
