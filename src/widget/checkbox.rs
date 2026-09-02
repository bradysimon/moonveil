//! Checkboxes using Moonveil's concrete theme and resolved control tokens.

use crate::{
    Theme,
    token::{Interaction, Surface},
};
use iced_core::{Background, Border};

pub use iced_widget::checkbox::{Catalog, Icon, Status, Style};

/// A custom checkbox style function using Moonveil's concrete theme.
pub type StyleFn<'a> = iced_widget::checkbox::StyleFn<'a, Theme>;

/// An Iced checkbox using Moonveil's concrete theme.
pub type Checkbox<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Checkbox<'a, Message, Theme, Renderer>;

/// Creates a checkbox using Moonveil's concrete theme.
pub fn checkbox<'a, Message, Renderer>(is_checked: bool) -> Checkbox<'a, Message, Renderer>
where
    Renderer: iced_core::text::Renderer,
{
    Checkbox::new(is_checked)
}

/// A built-in Moonveil checkbox style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// A standard binary control.
    #[default]
    Standard,
    /// A control whose value is invalid.
    Invalid,
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

/// Returns the resolved style for a built-in checkbox variant and status.
pub fn appearance(theme: &Theme, status: Status, variant: Variant) -> Style {
    let is_checked = match status {
        Status::Active { is_checked }
        | Status::Hovered { is_checked }
        | Status::Disabled { is_checked } => is_checked,
    };

    if matches!(status, Status::Disabled { .. }) {
        return Style {
            background: Background::Color(theme.surface(Surface::Field).into()),
            icon_color: theme.colors().content.disabled.into(),
            border: control_border(theme, theme.colors().borders.subtle),
            text_color: Some(theme.colors().content.disabled.into()),
        };
    }

    if variant == Variant::Standard && is_checked {
        let pair = match status {
            Status::Active { .. } => theme.colors().accent.solid.active,
            Status::Hovered { .. } => theme.colors().accent.solid.hovered,
            Status::Disabled { .. } => unreachable!(),
        };

        return Style {
            background: Background::Color(pair.color.into()),
            icon_color: pair.text.into(),
            border: control_border(theme, theme.colors().accent.border),
            text_color: Some(theme.colors().content.secondary.into()),
        };
    }

    let background = match status {
        Status::Active { .. } => theme.surface(Surface::Field),
        Status::Hovered { .. } => theme.interaction_on(Surface::Field, Interaction::Hover),
        Status::Disabled { .. } => unreachable!(),
    };
    let (icon_color, border_color) = match variant {
        Variant::Standard => (
            theme.colors().accent.indicator,
            theme.colors().borders.strong,
        ),
        Variant::Invalid => (
            theme.colors().danger.indicator,
            theme.colors().danger.border,
        ),
    };

    Style {
        background: Background::Color(background.into()),
        icon_color: icon_color.into(),
        border: control_border(theme, border_color),
        text_color: Some(theme.colors().content.secondary.into()),
    }
}

fn control_border(theme: &Theme, color: crate::Color) -> Border {
    Border {
        color: color.into(),
        width: theme.appearance().border.control,
        radius: theme.appearance().radius.xs.into(),
    }
}
