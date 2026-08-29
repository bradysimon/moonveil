//! A button using Moonveil's concrete theme and resolved tokens.

use crate::{
    Color, Element, Theme,
    token::{Fill, Pair, Semantic},
};
use iced_core::{Background, Border};

pub use crate::token::Intent;
pub use iced_widget::button::{Catalog, Status, Style};

/// A custom button style function using Moonveil's concrete theme.
pub type StyleFn<'a> = iced_widget::button::StyleFn<'a, Theme>;

/// An Iced button using Moonveil's concrete theme.
pub type Button<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Button<'a, Message, Theme, Renderer>;

/// Creates a button using Moonveil's concrete theme.
pub fn button<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Button<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    Button::new(content)
}

/// The visual treatment of a semantic button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticStyle {
    /// A high-emphasis semantic fill.
    Solid,
    /// A low-emphasis semantic fill.
    Soft,
    /// Semantic content and border on the surrounding surface.
    Outline,
    /// Semantic content without a persistent fill or border.
    Ghost,
}

/// A built-in Moonveil button style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// The primary action, mapped to accent solid tokens.
    #[default]
    Primary,
    /// A lower-emphasis action, mapped to accent soft tokens.
    Secondary,
    /// An accent action with a persistent border and no resting fill.
    Outline,
    /// An accent action without a persistent border or resting fill.
    Ghost,
    /// An action carrying explicit semantic intent and treatment.
    Semantic {
        intent: Intent,
        style: SemanticStyle,
    },
    /// A neutral icon action.
    Icon,
    /// A selected icon action, mapped to accent soft tokens.
    IconSelected,
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

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        match class {
            Class::Variant(variant) => appearance(self, *variant, status),
            Class::Custom(style) => style(self, status),
        }
    }
}

/// Returns the resolved style for a built-in button variant and status.
pub fn appearance(theme: &Theme, variant: Variant, status: Status) -> Style {
    if status == Status::Disabled {
        return disabled(theme, has_border(variant));
    }

    match variant {
        Variant::Primary => {
            let semantic = theme.colors().accent;
            filled(theme, semantic.solid, semantic.border, status)
        }
        Variant::Secondary | Variant::IconSelected => {
            let semantic = theme.colors().accent;
            filled(theme, semantic.soft, semantic.border, status)
        }
        Variant::Outline => outlined(
            theme,
            theme.colors().accent.foreground,
            Some(theme.colors().accent.border),
            status,
        ),
        Variant::Ghost => outlined(theme, theme.colors().accent.foreground, None, status),
        Variant::Semantic { intent, style } => {
            semantic_appearance(theme, semantic(theme, intent), style, status)
        }
        Variant::Icon => outlined(theme, theme.colors().content.secondary, None, status),
    }
}

fn semantic_appearance(
    theme: &Theme,
    semantic: Semantic,
    treatment: SemanticStyle,
    status: Status,
) -> Style {
    match treatment {
        SemanticStyle::Solid => filled(theme, semantic.solid, semantic.border, status),
        SemanticStyle::Soft => filled(theme, semantic.soft, semantic.border, status),
        SemanticStyle::Outline => {
            outlined(theme, semantic.foreground, Some(semantic.border), status)
        }
        SemanticStyle::Ghost => outlined(theme, semantic.foreground, None, status),
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

fn filled(theme: &Theme, fill: Fill, border_color: Color, status: Status) -> Style {
    let pair = pair(fill, status);

    Style {
        background: Some(Background::Color(pair.color.into())),
        text_color: pair.text.into(),
        border: control_border(theme, border_color, theme.appearance().border.control),
        ..Style::default()
    }
}

fn outlined(
    theme: &Theme,
    text_color: Color,
    border_color: Option<Color>,
    status: Status,
) -> Style {
    let background = match status {
        Status::Active | Status::Disabled => None,
        Status::Hovered => Some(Background::Color(theme.colors().interaction.hover.into())),
        Status::Pressed => Some(Background::Color(theme.colors().interaction.pressed.into())),
    };
    let (border_color, border_width) = border_color
        .map_or((iced_core::Color::TRANSPARENT, 0.0), |color| {
            (color.into(), theme.appearance().border.control)
        });

    Style {
        background,
        text_color: text_color.into(),
        border: Border {
            color: border_color,
            width: border_width,
            radius: theme.appearance().radius.sm.into(),
        },
        ..Style::default()
    }
}

fn disabled(theme: &Theme, has_border: bool) -> Style {
    Style {
        text_color: theme.colors().content.disabled.into(),
        border: control_border(
            theme,
            theme.colors().borders.subtle,
            if has_border {
                theme.appearance().border.hairline
            } else {
                0.0
            },
        ),
        ..Style::default()
    }
}

fn has_border(variant: Variant) -> bool {
    match variant {
        Variant::Primary
        | Variant::Secondary
        | Variant::Outline
        | Variant::IconSelected
        | Variant::Semantic {
            style: SemanticStyle::Solid | SemanticStyle::Soft | SemanticStyle::Outline,
            ..
        } => true,
        Variant::Ghost
        | Variant::Icon
        | Variant::Semantic {
            style: SemanticStyle::Ghost,
            ..
        } => false,
    }
}

fn pair(fill: Fill, status: Status) -> Pair {
    match status {
        Status::Active | Status::Disabled => fill.active,
        Status::Hovered => fill.hovered,
        Status::Pressed => fill.pressed,
    }
}

fn control_border(theme: &Theme, color: Color, width: f32) -> Border {
    Border {
        color: color.into(),
        width,
        radius: theme.appearance().radius.sm.into(),
    }
}
