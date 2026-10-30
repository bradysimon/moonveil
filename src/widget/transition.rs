//! Animate content between values.

use crate::{Element, Theme};
use iced_core::time::Instant;

pub use iced_widget::transition::{Program, reset, reset_raw};

pub type Transition<'a, Message, P, Renderer = iced_widget::Renderer> =
    iced_widget::transition::Transition<'a, Message, Theme, Renderer, P>;

/// Creates a [`Transition`] that animates toward `value` using the program built by `init`.
pub fn transition<'a, Message, Renderer, P, E>(
    value: P::Value,
    init: impl Fn() -> P + 'a,
    view: impl Fn(&P, Instant) -> E + 'a,
) -> Transition<'a, Message, P, Renderer>
where
    Renderer: iced_core::Renderer,
    P: Program,
    E: Into<Element<'a, Message, Renderer>>,
{
    Transition::new(init, value, view)
}
