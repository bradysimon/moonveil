//! Creates a responsive layout that produces elements based on available space.

use iced_core::Size;

pub type Responsive<'a, W> = iced_widget::Responsive<'a, W>;

/// Creates a new [`Responsive`] widget with a closure that produces its contents.
///
/// The view closure will receive the maximum available space for the [`Responsive`] during layout.
/// You can use this Size to conditionally build the contents.
pub fn responsive<'a, Content>(view: impl Fn(Size) -> Content + 'a) -> Responsive<'a, Content> {
    Responsive::new(view)
}
