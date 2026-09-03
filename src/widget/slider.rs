//! Sliders using Moonveil's concrete theme and resolved indicator tokens.

use std::ops::RangeInclusive;

use crate::{
    Theme,
    token::{Intent, Semantic, Surface},
};
use iced_core::{Background, Border};

pub use iced_widget::slider::{Catalog, Handle, HandleShape, Rail, Status, Style};

const RAIL_WIDTH: f32 = 4.0;
const HANDLE_RADIUS: f32 = 7.0;

/// A custom slider style function using Moonveil's concrete theme.
pub type StyleFn<'a> = iced_widget::slider::StyleFn<'a, Theme>;

/// An Iced slider using Moonveil's concrete theme.
pub type Slider<'a, Value, Message> = iced_widget::Slider<'a, Value, Message, Theme>;

/// Creates a slider using Moonveil's concrete theme.
pub fn slider<'a, Value, Message>(
    range: RangeInclusive<Value>,
    value: Value,
    on_change: impl Fn(Value) -> Message + 'a,
) -> Slider<'a, Value, Message>
where
    Value: Copy + PartialOrd,
    Message: Clone,
{
    Slider::new(range, value, on_change)
}

/// A built-in Moonveil slider style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// A neutral track with the accent indicator.
    #[default]
    Standard,
    /// A range control carrying explicit semantic intent.
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

/// Returns the resolved style for a built-in slider variant and status.
pub fn appearance(theme: &Theme, status: Status, variant: Variant) -> Style {
    let semantic = match variant {
        Variant::Standard => theme.colors().accent,
        Variant::Semantic(intent) => semantic(theme, intent),
    };
    let (handle_border_width, handle_border_color) = match status {
        Status::Active => (0.0, iced_core::Color::TRANSPARENT),
        Status::Hovered => (theme.appearance().border.control, semantic.border.into()),
        Status::Dragged => (
            theme.appearance().border.focus,
            theme.colors().borders.focus.into(),
        ),
    };

    Style {
        rail: Rail {
            backgrounds: (
                Background::Color(semantic.indicator.into()),
                Background::Color(theme.surface(Surface::Inset).into()),
            ),
            width: RAIL_WIDTH,
            border: Border {
                radius: theme.appearance().radius.full.into(),
                ..Border::default()
            },
        },
        handle: Handle {
            shape: HandleShape::Circle {
                radius: HANDLE_RADIUS,
            },
            background: Background::Color(semantic.indicator.into()),
            border_width: handle_border_width,
            border_color: handle_border_color,
        },
    }
}

fn semantic(theme: &Theme, intent: Intent) -> Semantic {
    match intent {
        Intent::Accent => theme.colors().accent,
        Intent::Success => theme.colors().success,
        Intent::Warning => theme.colors().warning,
        Intent::Danger => theme.colors().danger,
        Intent::Info => theme.colors().info,
    }
}
