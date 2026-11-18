pub type MouseArea<'a, Message, W> = iced_widget::MouseArea<'a, Message, W>;

/// Emit messages on mouse events.
pub fn mouse_area<'a, Message, W>(content: W) -> MouseArea<'a, Message, W> {
    MouseArea::new(content)
}
