//! Distribute content horizontally.

pub type Row<W> = iced_widget::Row<W>;

/// A container that distributes its contents horizontally.
pub fn row<W>(children: impl IntoIterator<Item = W>) -> Row<W>
where
    W: iced_core::widget::Meta,
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
        $crate::widget::Row::with_children([$($crate::Widget::boxed($x)),+])
    );
}
