//! Creates a responsive layout that produces elements based on available space.

use crate::{Element, Theme};
use iced_core::Size;

pub type Responsive<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Responsive<'a, Message, Theme, Renderer>;

/// Creates a new [`Responsive`] widget with a closure that produces its contents.
///
/// The view closure will receive the maximum available space for the [`Responsive`] during layout.
/// You can use this Size to conditionally build the contents.
pub fn responsive<'a, Message, Renderer, Content>(
    view: impl Fn(Size) -> Content + 'a,
) -> Responsive<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
    Content: Into<Element<'a, Message, Renderer>>,
{
    Responsive::new(view)
}
