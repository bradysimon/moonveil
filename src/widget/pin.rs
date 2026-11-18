//! Position content at fixed coordinates inside its bounds.

pub type Pin<W> = iced_widget::Pin<W>;

/// Creates a [`Pin`] containing an element.
pub fn pin<W>(content: W) -> Pin<W> {
    Pin::new(content)
}
