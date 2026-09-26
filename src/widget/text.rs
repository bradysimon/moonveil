//! Text using Moonveil's concrete theme and resolved content tokens.

use crate::{Theme, token::Intent};

pub use iced_widget::text::{
    Alignment, Catalog, Ellipsis, IntoFragment, LineHeight, Position, Shaping, Style, Wrapping,
};

/// Text sizes measured in logical pixels.
pub mod size {
    /// Compact annotations, table headers, and incidental metadata.
    pub const CAPTION: f32 = 11.0;
    /// Control labels and secondary interface copy.
    pub const LABEL: f32 = 12.0;
    /// Default body copy and control text.
    pub const BODY: f32 = 14.0;
    /// Compact section and panel titles.
    pub const TITLE: f32 = 16.0;
    /// Page and major section headings.
    pub const HEADING: f32 = 20.0;
    /// Reserved for the largest title in a view.
    pub const DISPLAY: f32 = 24.0;
}

/// A custom text style function using Moonveil's concrete theme.
pub type StyleFn<'a> = iced_widget::text::StyleFn<'a, Theme>;

/// Iced text using Moonveil's concrete theme.
pub type Text<'a> = iced_widget::Text<'a, Theme>;

/// Creates text using Moonveil's concrete theme.
pub fn text<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    Text::new(content).size(size::BODY)
}

/// Adds Moonveil's named variants to Iced text.
pub trait TextExt: Sized {
    /// Sets a built-in Moonveil text variant.
    fn class(self, variant: Variant) -> Self;
}

impl TextExt for Text<'_> {
    fn class(self, variant: Variant) -> Self {
        self.style(move |theme| appearance(theme, variant))
    }
}

/// A built-in Moonveil text style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// Inherits the foreground color from the surrounding renderer or container.
    #[default]
    Inherited,
    /// Headings, values, and primary document content.
    Primary,
    /// Body copy, labels, active controls, and icons.
    Secondary,
    /// Readable metadata, hints, and placeholders.
    Muted,
    /// Nonessential icons and incidental text.
    Decorative,
    /// Unavailable controls and values.
    Disabled,
    /// Standalone text carrying semantic intent.
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

/// Returns the resolved style for a built-in text variant.
pub fn appearance(theme: &Theme, variant: Variant) -> Style {
    let color = match variant {
        Variant::Inherited => None,
        Variant::Primary => Some(theme.colors().content.primary),
        Variant::Secondary => Some(theme.colors().content.secondary),
        Variant::Muted => Some(theme.colors().content.muted),
        Variant::Decorative => Some(theme.colors().content.decorative),
        Variant::Disabled => Some(theme.colors().content.disabled),
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
