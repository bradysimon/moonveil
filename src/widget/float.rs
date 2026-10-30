//! Make elements float over other content with Moonveil elevation shadows.

use crate::{Element, Theme};

pub use iced_widget::float::{Catalog, Style};

pub type StyleFn<'a> = iced_widget::float::StyleFn<'a, Theme>;

pub type Float<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Float<'a, Message, Theme, Renderer>;

/// Creates a [`Float`] containing an element.
pub fn float<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Float<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    Float::new(content)
}

/// A built-in Moonveil float style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// No shadow.
    Flat,
    /// The raised elevation shadow.
    Raised,
    /// The popover elevation shadow.
    #[default]
    Popover,
    /// The modal elevation shadow.
    Modal,
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

/// Returns the resolved style for a built-in float variant.
pub fn appearance(theme: &Theme, variant: Variant) -> Style {
    let shadows = theme.appearance().shadow;
    let shadow = match variant {
        Variant::Flat => return Style::default(),
        Variant::Raised => shadows.raised,
        Variant::Popover => shadows.popover,
        Variant::Modal => shadows.modal,
    };

    Style {
        shadow: shadow.into(),
        shadow_border_radius: theme.appearance().radius.md.into(),
    }
}
