//! Columns containing elements that use Moonveil's concrete theme.

pub type Column<W> = iced_widget::Column<W>;

/// Creates a column from an iterator of widgets.
pub fn column<W>(children: impl IntoIterator<Item = W>) -> Column<W>
where
    W: iced_core::widget::Meta,
{
    Column::with_children(children)
}

/// Creates a [`Column`] with the given children.
///
/// Columns distribute their children vertically.
#[macro_export]
macro_rules! column {
    () => (
        $crate::widget::Column::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::Column::with_children([$($crate::Widget::boxed($x)),+])
    );
}
