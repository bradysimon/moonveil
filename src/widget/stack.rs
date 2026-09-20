//! Display content on top of other content.

use crate::{Element, Theme};

pub type Stack<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Stack<'a, Message, Theme, Renderer>;

/// Creates a [`Stack`] from an iterator of elements.
pub fn stack<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Renderer>>,
) -> Stack<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    Stack::with_children(children)
}

/// Prevents mouse events from passing through an element to lower stack layers.
pub fn opaque<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_core::Renderer + 'a,
{
    iced_widget::opaque(content)
}
