//! Canvases draw interactive 2D graphics.
//!
//! [`Program`] defaults its theme to Iced's built-in theme, so implement
//! `Program<Message, moonveil::Theme>` explicitly.

use crate::Theme;

pub use iced_widget::canvas::*;

pub type Canvas<P, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Canvas<P, Message, Theme, Renderer>;

/// Creates a [`Canvas`] drawing the given program.
pub fn canvas<P, Message, Renderer>(program: P) -> Canvas<P, Message, Renderer>
where
    Renderer: iced_widget::graphics::geometry::Renderer,
    P: Program<Message, Theme, Renderer>,
{
    Canvas::new(program)
}
