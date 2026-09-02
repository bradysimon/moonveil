//! SVG images using Moonveil's concrete theme and resolved content tokens.

use crate::{Theme, token::Intent};

pub use iced_widget::svg::{Catalog, Handle, Status, Style};

/// A custom SVG style function using Moonveil's concrete theme.
pub type StyleFn<'a> = iced_widget::svg::StyleFn<'a, Theme>;

/// An Iced SVG image using Moonveil's concrete theme.
pub type Svg<'a> = iced_widget::Svg<'a, Theme>;

/// Creates an SVG image using Moonveil's concrete theme.
pub fn svg<'a>(handle: impl Into<Handle>) -> Svg<'a> {
    Svg::new(handle)
}

/// A built-in Moonveil SVG style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// Preserves the source artwork's original colors.
    #[default]
    Original,
    /// A symbolic icon using primary content color.
    Primary,
    /// A symbolic icon using secondary content color.
    Secondary,
    /// A symbolic icon using muted content color.
    Muted,
    /// A symbolic icon carrying explicit semantic intent.
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

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        match class {
            Class::Variant(variant) => appearance(self, status, *variant),
            Class::Custom(style) => style(self, status),
        }
    }
}

/// Returns the resolved style for a built-in SVG variant and status.
pub fn appearance(theme: &Theme, status: Status, variant: Variant) -> Style {
    let color = match variant {
        Variant::Original => None,
        Variant::Primary => Some(theme.colors().content.primary),
        Variant::Secondary => Some(match status {
            Status::Idle => theme.colors().content.secondary,
            Status::Hovered => theme.colors().content.primary,
        }),
        Variant::Muted => Some(match status {
            Status::Idle => theme.colors().content.muted,
            Status::Hovered => theme.colors().content.secondary,
        }),
        Variant::Semantic(intent) => Some(match intent {
            Intent::Accent => theme.colors().accent.foreground,
            Intent::Success => theme.colors().success.foreground,
            Intent::Warning => theme.colors().warning.foreground,
            Intent::Danger => theme.colors().danger.foreground,
            Intent::Info => theme.colors().info.foreground,
        }),
    };

    Style {
        color: color.map(Into::into),
    }
}
