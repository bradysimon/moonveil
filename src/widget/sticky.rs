//! Keep content in view while its scrollable ancestor scrolls past it.

pub type Sticky<W> = iced_widget::Sticky<W>;

/// Creates a [`Sticky`] containing an element.
pub fn sticky<W>(content: W) -> Sticky<W> {
    Sticky::new(content)
}
