//! Distribute content on a grid.

pub use iced_widget::grid::{Sizing, aspect_ratio};

pub type Grid<W> = iced_widget::Grid<W>;

/// Creates a grid from an iterator of widgets.
pub fn grid<W>(children: impl IntoIterator<Item = W>) -> Grid<W>
where
    W: iced_core::widget::Meta,
{
    Grid::with_children(children)
}
