//! Radio buttons using Moonveil's concrete theme and resolved control tokens.

use crate::{
    Theme,
    token::{Interaction, Surface},
};
use iced_core::Background;

pub use iced_widget::radio::{Catalog, Status, Style};

/// A custom radio style function using Moonveil's concrete theme.
pub type StyleFn<'a> = iced_widget::radio::StyleFn<'a, Theme>;

/// An Iced radio button using Moonveil's concrete theme.
pub type Radio<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Radio<'a, Message, Theme, Renderer>;

/// Creates a radio button using Moonveil's concrete theme.
pub fn radio<'a, Message, Renderer, Value>(
    label: impl Into<String>,
    value: Value,
    selected: Option<Value>,
    on_click: impl FnOnce(Value) -> Message,
) -> Radio<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::text::Renderer,
    Value: Copy + Eq,
{
    Radio::new(label, value, selected, on_click)
}

/// A built-in Moonveil radio style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// A standard single-choice control.
    #[default]
    Standard,
    /// A control whose selection is invalid.
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

/// Returns the resolved style for a built-in radio variant and status.
pub fn appearance(theme: &Theme, status: Status, variant: Variant) -> Style {
    let is_selected = match status {
        Status::Active { is_selected } | Status::Hovered { is_selected } => is_selected,
    };
    let background = match status {
        Status::Active { .. } => theme.surface(Surface::Field),
        Status::Hovered { .. } => theme.interaction_on(Surface::Field, Interaction::Hover),
    };
    let (dot_color, border_color) = match variant {
        Variant::Standard => (
            theme.colors().accent.indicator,
            if is_selected {
                theme.colors().borders.selected
            } else {
                theme.colors().borders.strong
            },
        ),
        Variant::Invalid => (
            theme.colors().danger.indicator,
            theme.colors().danger.border,
        ),
    };

    Style {
        background: Background::Color(background.into()),
        dot_color: dot_color.into(),
        border_width: theme.appearance().border.control,
        border_color: border_color.into(),
        text_color: Some(theme.colors().content.secondary.into()),
    }
}
