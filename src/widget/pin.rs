//! Position content at fixed coordinates inside its bounds.

use crate::{Element, Theme};

pub type Pin<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Pin<'a, Message, Theme, Renderer>;

/// Creates a [`Pin`] containing an element.
pub fn pin<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Pin<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    Pin::new(content)
}
