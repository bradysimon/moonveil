//! Animate content between values.

use iced_core::time::Instant;

pub use iced_widget::transition::{Program, reset, reset_raw};

pub type Transition<'a, Message, W, P> = iced_widget::transition::Transition<'a, Message, W, P>;

/// Creates a [`Transition`] that animates toward `value` using the program built by `init`.
pub fn transition<'a, Message, P, E>(
    value: P::Value,
    init: impl Fn() -> P + 'a,
    view: impl Fn(&P, Instant) -> E + 'a,
) -> Transition<'a, Message, E, P>
where
    P: Program,
{
    Transition::new(init, value, view)
}
