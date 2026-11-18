//! A custom theme and design framework for iced.
//!
//! This is a placeholder publish while awaiting the next Iced version so that
//! I can utilize the latest features on the `master` branch. I aim to release
//! a real version once iced 0.15 is released.
mod color;
mod contrast;
pub mod highlighter;
pub mod icon;
pub mod spacing;
mod theme;
pub mod token;
pub mod widget;

pub use color::Color;
pub use contrast::{Contrast, Targets};
pub use theme::{
    Appearance, BorderWidths, CategoricalSeeds, Definition, Depth, Elevation, Metadata, Polarity,
    Radii, Seed, Shadow, Shadows, Theme,
};
pub use token::ResolveError;

/// A generic Iced element using Moonveil's concrete [`Theme`].
pub type Element<'a, Message, Renderer = iced_widget::Renderer> =
    iced_core::Element<'a, Message, Theme, Renderer>;

/// A widget using Moonveil's concrete theme by default.
pub trait Widget<Message, Theme = crate::Theme, Renderer = iced_widget::Renderer>:
    iced_core::Widget<Message, Theme, Renderer>
{
    /// Maps the messages produced by this widget.
    fn map<F, Output>(self, mapper: F) -> iced_core::widget::Map<Self, F, Message>
    where
        Self: Sized,
        F: Fn(Message) -> Output,
    {
        self._map(mapper)
    }

    /// Boxes this widget into an element.
    fn boxed<'a>(self) -> iced_core::Element<'a, Message, Theme, Renderer>
    where
        Self: Sized + 'a,
    {
        self._boxed()
    }

    /// Explains this widget's layout graphically.
    fn explain(
        self,
        color: impl Into<iced_core::Color>,
    ) -> impl iced_core::Widget<Message, Theme, Renderer>
    where
        Self: Sized,
        Renderer: iced_core::Renderer,
    {
        iced_widget::Widget::explain(self, color)
    }
}

impl<Message, Theme, Renderer, W> Widget<Message, Theme, Renderer> for W where
    W: iced_core::Widget<Message, Theme, Renderer>
{
}
