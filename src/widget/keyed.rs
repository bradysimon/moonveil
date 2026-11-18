//! Keyed widgets keep continuity of their children's state across updates.

pub type Column<Key, W> = iced_widget::keyed::Column<Key, W>;

/// Creates a keyed column from an iterator of keyed widgets.
pub fn column<Key, W>(children: impl IntoIterator<Item = (Key, W)>) -> Column<Key, W>
where
    Key: Copy + PartialEq,
    W: iced_core::widget::Meta,
{
    Column::with_children(children)
}
