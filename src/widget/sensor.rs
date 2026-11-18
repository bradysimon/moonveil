//! Generate messages when content pops in and out of view.

pub type Sensor<'a, Key, Message, W> = iced_widget::Sensor<'a, Key, Message, W>;

/// Creates a [`Sensor`] containing an element.
pub fn sensor<'a, Message, W>(content: W) -> Sensor<'a, (), Message, W> {
    iced_widget::Sensor::new(content)
}
