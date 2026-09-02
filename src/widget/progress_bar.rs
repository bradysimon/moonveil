//! Progress bars using Moonveil's concrete theme and resolved indicator tokens.

use std::ops::RangeInclusive;

use crate::{
    Theme,
    token::{Intent, Surface},
};
use iced_core::{Background, Border};

pub use iced_widget::progress_bar::{Catalog, Style};

/// A custom progress bar style function using Moonveil's concrete theme.
pub type StyleFn<'a> = iced_widget::progress_bar::StyleFn<'a, Theme>;

/// An Iced progress bar using Moonveil's concrete theme.
pub type ProgressBar<'a> = iced_widget::ProgressBar<'a, Theme>;

/// Creates a progress bar using Moonveil's concrete theme.
pub fn progress_bar<'a>(range: RangeInclusive<f32>, value: f32) -> ProgressBar<'a> {
    ProgressBar::new(range, value)
}

/// A built-in Moonveil progress bar style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// A neutral track with the accent indicator.
    #[default]
    Standard,
    /// A progress indicator carrying explicit semantic intent.
    Semantic(Intent),
}

/// A built-in variant or downstream custom style function.
pub enum Class<'a> {
    /// A built-in, allocation-free Moonveil variant.
    Variant(Variant),
    /// A downstream custom style function.
    Custom(StyleFn<'a>),
}

impl Default for Class<'_> {
    fn default() -> Self {
        Self::Variant(Variant::default())
    }
}

impl<'a> From<Variant> for Class<'a> {
    fn from(variant: Variant) -> Self {
        Self::Variant(variant)
    }
}

impl<'a> From<StyleFn<'a>> for Class<'a> {
    fn from(style: StyleFn<'a>) -> Self {
        Self::Custom(style)
    }
}

impl Catalog for Theme {
    type Class<'a> = Class<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Class::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        match class {
            Class::Variant(variant) => appearance(self, *variant),
            Class::Custom(style) => style(self),
        }
    }
}

/// Returns the resolved style for a built-in progress bar variant.
pub fn appearance(theme: &Theme, variant: Variant) -> Style {
    let indicator = match variant {
        Variant::Standard => theme.colors().accent.indicator,
        Variant::Semantic(intent) => match intent {
            Intent::Accent => theme.colors().accent.indicator,
            Intent::Success => theme.colors().success.indicator,
            Intent::Warning => theme.colors().warning.indicator,
            Intent::Danger => theme.colors().danger.indicator,
            Intent::Info => theme.colors().info.indicator,
        },
    };

    Style {
        background: Background::Color(theme.surface(Surface::Inset).into()),
        bar: Background::Color(indicator.into()),
        border: Border {
            color: theme.colors().borders.standard.into(),
            width: theme.appearance().border.control,
            radius: theme.appearance().radius.full.into(),
        },
    }
}
