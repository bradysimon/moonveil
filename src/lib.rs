//! A custom theme and design framework for iced.
//!
//! This is a placeholder publish while awaiting the next Iced version so that
//! I can utilize the latest features on the `master` branch. I aim to release
//! a real version once iced 0.15 is released.
mod color;
mod contrast;
mod theme;
pub mod token;

pub use color::Color;
pub use contrast::{Profile, Targets};
pub use theme::{Definition, Metadata, Polarity, Seed, Theme};
pub use token::ResolveError;
