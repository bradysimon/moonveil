//! Distribute content horizontally.

use crate::{Element, Theme};

pub type Row<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Row<'a, Message, Theme, Renderer>;

/// A container that distributes its contents horizontally.
pub fn row<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Renderer>>,
) -> Row<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    Row::with_children(children)
}

/// Creates a [`Row`] with the given children.
///
/// Rows distribute their children horizontally.
#[macro_export]
macro_rules! row {
    () => (
        $crate::widget::Row::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::Row::with_children([$($crate::Element::from($x)),+])
    );
}
