use crate::{Element, Theme};

pub type MouseArea<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::MouseArea<'a, Message, Theme, Renderer>;

/// Emit messages on mouse events.
pub fn mouse_area<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> MouseArea<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    MouseArea::new(content)
}
