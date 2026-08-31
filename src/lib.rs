//! A custom theme and design framework for iced.
//!
//! This is a placeholder publish while awaiting the next Iced version so that
//! I can utilize the latest features on the `master` branch. I aim to release
//! a real version once iced 0.15 is released.
mod color;
mod contrast;
pub mod spacing;
mod theme;
pub mod token;
pub mod widget;

pub use color::Color;
pub use contrast::{Contrast, Targets};
pub use theme::{
    Appearance, BorderWidths, Definition, Metadata, Polarity, Radii, Seed, Shadow, Shadows, Theme,
};
pub use token::ResolveError;

/// A generic Iced element using Moonveil's concrete [`Theme`].
pub type Element<'a, Message, Renderer = iced_widget::Renderer> =
    iced_core::Element<'a, Message, Theme, Renderer>;
