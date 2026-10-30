//! Keyed widgets keep continuity of their children's state across updates.

use crate::{Element, Theme};

pub type Column<'a, Key, Message, Renderer = iced_widget::Renderer> =
    iced_widget::keyed::Column<'a, Key, Message, Theme, Renderer>;

/// Creates a keyed column from an iterator of keyed Moonveil elements.
pub fn column<'a, Key, Message, Renderer>(
    children: impl IntoIterator<Item = (Key, Element<'a, Message, Renderer>)>,
) -> Column<'a, Key, Message, Renderer>
where
    Key: Copy + PartialEq,
    Renderer: iced_core::Renderer,
{
    Column::with_children(children)
}
