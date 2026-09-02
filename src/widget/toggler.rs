//! Togglers using Moonveil's concrete theme and resolved control tokens.

use crate::{
    Theme,
    token::{Intent, Interaction, Semantic, Surface},
};
use iced_core::Background;

pub use iced_widget::toggler::{Catalog, Status, Style};

/// A custom toggler style function using Moonveil's concrete theme.
pub type StyleFn<'a> = iced_widget::toggler::StyleFn<'a, Theme>;

/// An Iced toggler using Moonveil's concrete theme.
pub type Toggler<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Toggler<'a, Message, Theme, Renderer>;

/// Recommended toggler sizes measured in logical pixels.
pub mod size {
    /// A compact toggler for dense interfaces.
    pub const COMPACT: f32 = 16.0;
    /// The standard Moonveil toggler size.
    pub const DEFAULT: f32 = 20.0;
}

/// Creates a toggler using Moonveil's concrete theme.
pub fn toggler<'a, Message, Renderer>(is_toggled: bool) -> Toggler<'a, Message, Renderer>
where
    Renderer: iced_core::text::Renderer,
{
    Toggler::new(is_toggled).size(size::DEFAULT)
}

/// A built-in Moonveil toggler style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// A standard binary preference.
    #[default]
    Standard,
    /// A control whose value is invalid.
    Invalid,
    /// A toggled state carrying explicit semantic intent.
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

/// Returns the resolved style for a built-in toggler variant and status.
pub fn appearance(theme: &Theme, status: Status, variant: Variant) -> Style {
    let is_toggled = match status {
        Status::Active { is_toggled }
        | Status::Hovered { is_toggled }
        | Status::Disabled { is_toggled } => is_toggled,
    };

    if matches!(status, Status::Disabled { .. }) {
        return style(
            theme,
            theme.surface(Surface::Inset),
            theme.colors().content.disabled,
            theme.colors().borders.subtle,
            theme.colors().content.disabled,
        );
    }

    let semantic = match variant {
        Variant::Standard => theme.colors().accent,
        Variant::Invalid => theme.colors().danger,
        Variant::Semantic(intent) => semantic(theme, intent),
    };
    let (track, thumb) = if is_toggled {
        let pair = match status {
            Status::Active { .. } => semantic.solid.active,
            Status::Hovered { .. } => semantic.solid.hovered,
            Status::Disabled { .. } => unreachable!(),
        };

        (pair.color, pair.text)
    } else {
        let track = match status {
            Status::Active { .. } => theme.surface(Surface::Inset),
            Status::Hovered { .. } => theme.interaction_on(Surface::Inset, Interaction::Hover),
            Status::Disabled { .. } => unreachable!(),
        };

        (track, theme.colors().content.secondary)
    };
    let border = match variant {
        Variant::Invalid => semantic.border,
        Variant::Semantic(_) if is_toggled => semantic.border,
        Variant::Standard if is_toggled => theme.colors().borders.selected,
        Variant::Standard | Variant::Semantic(_) => theme.colors().borders.strong,
    };

    style(
        theme,
        track,
        thumb,
        border,
        theme.colors().content.secondary,
    )
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

fn style(
    theme: &Theme,
    background: crate::Color,
    foreground: crate::Color,
    border: crate::Color,
    text: crate::Color,
) -> Style {
    Style {
        background: Background::Color(background.into()),
        background_border_width: theme.appearance().border.control,
        background_border_color: border.into(),
        foreground: Background::Color(foreground.into()),
        foreground_border_width: 0.0,
        foreground_border_color: iced_core::Color::TRANSPARENT,
        text_color: Some(text.into()),
        border_radius: None,
        padding_ratio: 0.12,
    }
}
