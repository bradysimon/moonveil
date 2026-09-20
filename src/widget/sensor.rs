//! Generate messages when content pops in and out of view.

use crate::{Element, Theme};

pub type Sensor<'a, Key, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Sensor<'a, Key, Message, Theme, Renderer>;

/// Creates a [`Sensor`] containing an element.
pub fn sensor<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Sensor<'a, (), Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    iced_widget::Sensor::new(content)
}
