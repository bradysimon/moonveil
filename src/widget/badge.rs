//! Badges are compact, non-interactive labels for small bits of data.

use crate::{
    Element, Theme,
    token::Intent,
    widget::{
        container::{self, Container},
        text::{self, IntoFragment, Text},
    },
};
use iced_core::{Alignment, Background, Border, Length, Padding};

pub use iced_widget::container::Style;

pub type StyleFn<'a> = container::StyleFn<'a>;

/// Creates a badge displaying a `label`.
pub fn badge<'a>(label: impl IntoFragment<'a>) -> Badge<'a> {
    Badge::new(label)
}

/// A compact label describing nearby content.
pub struct Badge<'a> {
    label: Text<'a>,
    class: Class<'a>,
    shape: Shape,
    size: Size,
    width: Length,
}

impl<'a> Badge<'a> {
    /// Creates a badge displaying `label`.
    pub fn new(label: impl IntoFragment<'a>) -> Self {
        Self {
            label: text::text(label),
            class: Class::default(),
            shape: Shape::default(),
            size: Size::default(),
            width: Length::Shrink,
        }
    }

    /// Sets the style of the badge.
    #[must_use]
    pub fn class(mut self, class: impl Into<Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets the corner treatment of built-in variants.
    #[must_use]
    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }

    /// Sets the text size and padding of the badge.
    #[must_use]
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Sets the width of the badge; the label is centered within it.
    ///
    /// A fixed width keeps badges of varying labels aligned in a list.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

/// The visual treatment of a semantic badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticStyle {
    /// A high-emphasis semantic fill.
    Solid,
    /// A low-emphasis semantic fill.
    Soft,
    /// Semantic content and border on the surrounding surface.
    Outline,
}

/// A built-in Moonveil badge style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// A low-emphasis neutral fill for counts and metadata.
    #[default]
    Neutral,
    /// A neutral border without a fill for types and categories.
    Outline,
    /// A badge carrying explicit semantic intent and treatment.
    Semantic {
        intent: Intent,
        style: SemanticStyle,
    },
}

/// The corner treatment of a badge.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Shape {
    /// Tight rounding suited to labels and statuses.
    #[default]
    Rounded,
    /// Fully rounded ends suited to counts and tags.
    Pill,
}

/// The text size and padding of a badge.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Size {
    Small,
    #[default]
    Medium,
    Large,
}

impl Size {
    /// The label text size in logical pixels.
    pub const fn text_size(self) -> f32 {
        match self {
            Self::Small => text::size::CAPTION,
            Self::Medium => text::size::LABEL,
            Self::Large => text::size::BODY,
        }
    }

    /// The padding between the label and the badge edge.
    pub fn padding(self) -> Padding {
        match self {
            Self::Small => Padding::from([1.0, 5.0]),
            Self::Medium => Padding::from([2.0, 6.0]),
            Self::Large => Padding::from([3.0, 8.0]),
        }
    }
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

/// Returns the resolved style for a built-in badge variant and shape.
pub fn appearance(theme: &Theme, variant: Variant, shape: Shape) -> Style {
    let colors = theme.colors();
    // Only unfilled variants draw a border so fills and outlines stay distinct.
    let (background, text_color, border_color) = match variant {
        // A translucent overlay stays visible on every surface, and content is validated against it.
        Variant::Neutral => (
            Some(colors.interaction.pressed),
            colors.content.secondary,
            None,
        ),
        Variant::Outline => (
            None,
            colors.content.secondary,
            Some(colors.borders.standard),
        ),
        Variant::Semantic { intent, style } => {
            let semantic = colors.semantic(intent);

            match style {
                SemanticStyle::Solid => (
                    Some(semantic.solid.active.color),
                    semantic.solid.active.text,
                    None,
                ),
                SemanticStyle::Soft => (
                    Some(semantic.soft.active.color),
                    semantic.soft.active.text,
                    None,
                ),
                SemanticStyle::Outline => (None, semantic.foreground, Some(semantic.border)),
            }
        }
    };
    let radius = match shape {
        Shape::Rounded => theme.appearance().radius.xs,
        Shape::Pill => theme.appearance().radius.full,
    };

    Style {
        text_color: Some(text_color.into()),
        background: background.map(|color| Background::Color(color.into())),
        border: Border {
            color: border_color.map_or(iced_core::Color::TRANSPARENT, Into::into),
            width: border_color.map_or(0.0, |_| theme.appearance().border.hairline),
            radius: radius.into(),
        },
        ..Style::default()
    }
}

impl<'a, Message, Renderer> From<Badge<'a>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_core::text::Renderer + 'a,
{
    fn from(badge: Badge<'a>) -> Self {
        let shape = badge.shape;
        let class = match badge.class {
            Class::Variant(variant) => container::Class::Custom(Box::new(move |theme: &Theme| {
                appearance(theme, variant, shape)
            })),
            Class::Custom(style) => container::Class::Custom(style),
        };

        Container::new(badge.label.size(badge.size.text_size()))
            .class(class)
            .padding(badge.size.padding())
            .width(badge.width)
            .align_x(Alignment::Center)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soft_semantic_uses_soft_pair() {
        let theme = Theme::default_dark();
        let style = appearance(
            &theme,
            Variant::Semantic {
                intent: Intent::Success,
                style: SemanticStyle::Soft,
            },
            Shape::Rounded,
        );
        let pair = theme.colors().success.soft.active;

        assert_eq!(style.background, Some(Background::Color(pair.color.into())));
        assert_eq!(style.text_color, Some(pair.text.into()));
    }

    #[test]
    fn outlines_have_no_fill() {
        let theme = Theme::default_dark();

        for variant in [
            Variant::Outline,
            Variant::Semantic {
                intent: Intent::Danger,
                style: SemanticStyle::Outline,
            },
        ] {
            assert_eq!(appearance(&theme, variant, Shape::Rounded).background, None);
        }
    }

    #[test]
    fn pill_uses_full_radius() {
        let theme = Theme::default_dark();
        let style = appearance(&theme, Variant::Neutral, Shape::Pill);

        assert_eq!(style.border.radius, theme.appearance().radius.full.into());
    }
}
