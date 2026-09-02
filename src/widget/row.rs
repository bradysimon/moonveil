//! Rows containing elements that use Moonveil's concrete theme.

use crate::{Element, Theme};

/// An Iced row containing elements that use Moonveil's concrete theme.
pub type Row<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Row<'a, Message, Theme, Renderer>;

/// Creates a row from an iterator of Moonveil elements.
pub fn row<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Renderer>>,
) -> Row<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    Row::with_children(children)
}
