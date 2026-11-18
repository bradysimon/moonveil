//! Display content on top of other content.

use crate::Theme;

pub type Stack<W> = iced_widget::Stack<W>;

/// Creates a [`Stack`] from an iterator of widgets.
pub fn stack<W>(children: impl IntoIterator<Item = W>) -> Stack<W>
where
    W: iced_core::widget::Meta,
{
    Stack::with_children(children)
}

/// Prevents mouse events from passing through an element to lower stack layers.
pub fn opaque<'a, Message, Renderer>(
    content: impl iced_core::Widget<Message, Theme, Renderer> + 'a,
) -> impl crate::Widget<Message, Theme, Renderer> + 'a
where
    Message: 'a,
    Renderer: iced_core::Renderer + 'a,
{
    iced_widget::opaque(content)
}

/// Displays `top` over `base` only while `base` is hovered.
pub fn hover<'a, Message, Renderer>(
    base: impl iced_core::Widget<Message, Theme, Renderer> + 'a,
    top: impl iced_core::Widget<Message, Theme, Renderer> + 'a,
) -> impl crate::Widget<Message, Theme, Renderer> + 'a
where
    Message: 'a,
    Renderer: iced_core::Renderer + 'a,
{
    iced_widget::hover(base, top)
}
