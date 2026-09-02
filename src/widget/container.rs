//! Containers using Moonveil's concrete theme and resolved surface tokens.

use crate::{
    Color, Element, Theme,
    token::{Intent, Surface},
};
use iced_core::{Background, Border};

pub use iced_widget::container::{Catalog, Style};

/// A custom container style function using Moonveil's concrete theme.
pub type StyleFn<'a> = iced_widget::container::StyleFn<'a, Theme>;

/// An Iced container using Moonveil's concrete theme.
pub type Container<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Container<'a, Message, Theme, Renderer>;

/// Creates a container using Moonveil's concrete theme.
pub fn container<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Container<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    Container::new(content)
}

/// The visual treatment of a semantic container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticStyle {
    /// A high-emphasis semantic fill.
    Solid,
    /// A low-emphasis semantic fill.
    Soft,
}

/// A built-in Moonveil container style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// A composition wrapper with no visual treatment.
    #[default]
    Transparent,
    /// Recessed content such as code wells and terminal output.
    Sunken,
    /// Receding application chrome such as sidebars.
    Canvas,
    /// Local recesses such as tracks and grouped control backgrounds.
    Inset,
    /// The primary content plane.
    Surface,
    /// Inline panels and grouped content without a shadow.
    Raised,
    /// Content placed above another plane.
    Overlay,
    /// Editable content and dense data planes.
    Field,
    /// A container carrying explicit semantic intent and treatment.
    Semantic {
        intent: Intent,
        style: SemanticStyle,
    },
    /// An anchored floating surface.
    Popover,
    /// A dialog surface placed above a separate scrim.
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

/// Returns the resolved style for a built-in container variant.
pub fn appearance(theme: &Theme, variant: Variant) -> Style {
    match variant {
        Variant::Transparent => Style::default(),
        Variant::Sunken => surface(
            theme,
            theme.surface(Surface::Sunken),
            Some(theme.colors().borders.standard),
        ),
        Variant::Canvas => surface(theme, theme.surface(Surface::Canvas), None),
        Variant::Inset => surface(
            theme,
            theme.surface(Surface::Inset),
            Some(theme.colors().borders.subtle),
        ),
        Variant::Surface => surface(theme, theme.surface(Surface::Surface), None),
        Variant::Raised => surface(
            theme,
            theme.surface(Surface::Raised),
            Some(theme.colors().borders.standard),
        ),
        Variant::Overlay => surface(
            theme,
            theme.surface(Surface::Overlay),
            Some(theme.colors().borders.standard),
        ),
        Variant::Field => field(theme),
        Variant::Semantic { intent, style } => semantic(theme, intent, style),
        Variant::Popover => Style {
            shadow: theme.appearance().shadow.popover.into(),
            ..surface(
                theme,
                theme.surface(Surface::Overlay),
                Some(theme.colors().borders.standard),
            )
        },
        Variant::Modal => Style {
            shadow: theme.appearance().shadow.modal.into(),
            border: Border {
                radius: theme.appearance().radius.lg.into(),
                ..control_border(theme, theme.colors().borders.standard)
            },
            ..surface(theme, theme.surface(Surface::Overlay), None)
        },
    }
}

fn surface(theme: &Theme, background: Color, border: Option<Color>) -> Style {
    Style {
        text_color: Some(theme.colors().content.primary.into()),
        background: Some(Background::Color(background.into())),
        border: border.map_or_else(Border::default, |color| Border {
            color: color.into(),
            width: theme.appearance().border.hairline,
            radius: theme.appearance().radius.md.into(),
        }),
        ..Style::default()
    }
}

fn field(theme: &Theme) -> Style {
    Style {
        text_color: Some(theme.colors().content.primary.into()),
        background: Some(Background::Color(theme.surface(Surface::Field).into())),
        border: control_border(theme, theme.colors().borders.strong),
        ..Style::default()
    }
}

fn semantic(theme: &Theme, intent: Intent, style: SemanticStyle) -> Style {
    let semantic = match intent {
        Intent::Accent => theme.colors().accent,
        Intent::Success => theme.colors().success,
        Intent::Warning => theme.colors().warning,
        Intent::Danger => theme.colors().danger,
        Intent::Info => theme.colors().info,
    };
    let pair = match style {
        SemanticStyle::Solid => semantic.solid.active,
        SemanticStyle::Soft => semantic.soft.active,
    };

    Style {
        text_color: Some(pair.text.into()),
        background: Some(Background::Color(pair.color.into())),
        border: control_border(theme, semantic.border),
        ..Style::default()
    }
}

fn control_border(theme: &Theme, color: Color) -> Border {
    Border {
        color: color.into(),
        width: theme.appearance().border.control,
        radius: theme.appearance().radius.sm.into(),
    }
}
