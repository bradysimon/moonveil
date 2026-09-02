//! Columns containing elements that use Moonveil's concrete theme.

use crate::{Element, Theme};

/// An Iced column containing elements that use Moonveil's concrete theme.
pub type Column<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Column<'a, Message, Theme, Renderer>;

/// Creates a column from an iterator of Moonveil elements.
pub fn column<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Renderer>>,
) -> Column<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    Column::with_children(children)
}
