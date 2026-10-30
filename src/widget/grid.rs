//! Distribute content on a grid.

use crate::{Element, Theme};

pub use iced_widget::grid::{Sizing, aspect_ratio};

pub type Grid<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Grid<'a, Message, Theme, Renderer>;

/// Creates a grid from an iterator of Moonveil elements.
pub fn grid<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Renderer>>,
) -> Grid<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    Grid::with_children(children)
}
