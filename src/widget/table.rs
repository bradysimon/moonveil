//! Display tables.

use crate::{Element, Theme};
use iced_core::Background;

pub use iced_widget::table::{Catalog, Column, Style};

pub type StyleFn<'a> = iced_widget::table::StyleFn<'a, Theme>;

pub type Table<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::table::Table<'a, Message, Theme, Renderer>;

/// Creates a new [`Table`] with the given columns and rows.
///
/// Columns can be created using the [`column()`] function, while rows can be any
/// iterator over some data type `T`.
pub fn table<'a, 'b, T, Message, Renderer>(
    columns: impl IntoIterator<Item = Column<'a, 'b, T, Message, Theme, Renderer>>,
    rows: impl IntoIterator<Item = T>,
) -> Table<'a, Message, Renderer>
where
    T: Clone,
    Renderer: iced_core::Renderer,
{
    iced_widget::table::table(columns, rows)
}

/// Creates a new [`Column`] with the given header and view function.
///
/// The view function will be called for each row in a [`Table`] and it must
/// produce the resulting contents of a cell.
pub fn column<'a, 'b, T, E, Message, Renderer>(
    header: impl Into<Element<'a, Message, Renderer>>,
    view: impl Fn(T) -> E + 'b,
) -> Column<'a, 'b, T, Message, Theme, Renderer>
where
    T: 'a,
    E: Into<Element<'a, Message, Renderer>>,
{
    iced_widget::table::column(header, view)
}

/// A built-in Moonveil table style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// Separators with ordinary visibility.
    #[default]
    Standard,
    /// Low-emphasis separators for dense data.
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

/// Returns the resolved style for a built-in table variant.
pub fn appearance(theme: &Theme, variant: Variant) -> Style {
    let color = match variant {
        Variant::Standard => theme.colors().borders.standard,
        Variant::Subtle => theme.colors().borders.subtle,
    };

    let separator = Background::Color(color.into());

    Style {
        separator_x: separator,
        separator_y: separator,
    }
}
