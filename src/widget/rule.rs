//! Rules using Moonveil's concrete theme and resolved border tokens.

use crate::{Theme, token::Intent};

pub use iced_widget::rule::{Catalog, FillMode, Style};

/// A custom rule style function using Moonveil's concrete theme.
pub type StyleFn<'a> = iced_widget::rule::StyleFn<'a, Theme>;

/// An Iced rule using Moonveil's concrete theme.
pub type Rule<'a> = iced_widget::Rule<'a, Theme>;

/// Creates a horizontal rule using Moonveil's concrete theme.
pub fn horizontal<'a>(height: impl Into<iced_core::Pixels>) -> Rule<'a> {
    iced_widget::rule::horizontal(height)
}

/// Creates a vertical rule using Moonveil's concrete theme.
pub fn vertical<'a>(width: impl Into<iced_core::Pixels>) -> Rule<'a> {
    iced_widget::rule::vertical(width)
}

/// A built-in Moonveil rule style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// A decorative, low-emphasis separator.
    Subtle,
    /// An ordinary visible separator.
    #[default]
    Standard,
    /// A strongly emphasized separator.
    Strong,
    /// A separator carrying explicit semantic intent.
    Semantic(Intent),
}

/// A built-in variant or custom style function.
pub enum Class<'a> {
    /// A built-in variant.
    Variant(Variant),
    /// A custom style function.
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

/// Returns the resolved style for a built-in rule variant.
pub fn appearance(theme: &Theme, variant: Variant) -> Style {
    let color = match variant {
        Variant::Subtle => theme.colors().borders.subtle,
        Variant::Standard => theme.colors().borders.standard,
        Variant::Strong => theme.colors().borders.strong,
        Variant::Semantic(intent) => match intent {
            Intent::Accent => theme.colors().accent.border,
            Intent::Success => theme.colors().success.border,
            Intent::Warning => theme.colors().warning.border,
            Intent::Danger => theme.colors().danger.border,
            Intent::Info => theme.colors().info.border,
        },
    };

    Style {
        color: color.into(),
        radius: theme.appearance().radius.full.into(),
        fill_mode: FillMode::Full,
        snap: true,
    }
}
