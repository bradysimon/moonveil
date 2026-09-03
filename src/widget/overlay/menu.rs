//! Menu overlays using Moonveil's concrete theme and resolved overlay tokens.

use crate::{
    Theme,
    token::{Interaction, Surface},
};
use iced_core::{Background, Border};

pub use iced_widget::overlay::menu::{Catalog, Style};

/// A custom menu style function using Moonveil's concrete theme.
pub type StyleFn<'a> = iced_widget::overlay::menu::StyleFn<'a, Theme>;

/// A built-in Moonveil menu style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// A floating menu with an ordinary visible edge.
    #[default]
    Standard,
    /// A floating menu with a quieter edge.
    Subtle,
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

    fn default<'a>() -> <Self as Catalog>::Class<'a> {
        Class::default()
    }

    fn style(&self, class: &<Self as Catalog>::Class<'_>) -> Style {
        match class {
            Class::Variant(variant) => appearance(self, *variant),
            Class::Custom(style) => style(self),
        }
    }
}

/// Returns the resolved style for a built-in menu variant.
pub fn appearance(theme: &Theme, variant: Variant) -> Style {
    let (border_color, border_width) = match variant {
        Variant::Standard => (
            theme.colors().borders.standard,
            theme.appearance().border.control,
        ),
        Variant::Subtle => (
            theme.colors().borders.subtle,
            theme.appearance().border.hairline,
        ),
    };

    Style {
        background: Background::Color(theme.surface(Surface::Overlay).into()),
        border: Border {
            color: border_color.into(),
            width: border_width,
            radius: theme.appearance().radius.sm.into(),
        },
        text_color: theme.colors().content.primary.into(),
        selected_text_color: theme.colors().content.primary.into(),
        selected_background: Background::Color(
            theme
                .interaction_on(Surface::Overlay, Interaction::Selected)
                .into(),
        ),
        shadow: theme.appearance().shadow.popover.into(),
    }
}
