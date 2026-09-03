//! Scrollable regions using Moonveil's concrete theme and resolved surface tokens.

use crate::{Element, Theme, token::Surface};
use iced_core::{Background, Border};

pub use iced_core::widget::Id;
pub use iced_widget::scrollable::{
    AbsoluteOffset, Anchor, AutoScroll, Catalog, Direction, Rail, RelativeOffset, Scrollbar,
    Scroller, Status, Style, Viewport,
};

/// A custom scrollable style function using Moonveil's concrete theme.
pub type StyleFn<'a> = iced_widget::scrollable::StyleFn<'a, Theme>;

/// An Iced scrollable region using Moonveil's concrete theme.
pub type Scrollable<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Scrollable<'a, Message, Theme, Renderer>;

const SCROLLBAR_WIDTH: f32 = 8.0;

/// Creates a scrollbar using Moonveil's default geometry.
pub fn scrollbar() -> Scrollbar {
    Scrollbar::new()
        .width(SCROLLBAR_WIDTH)
        .scroller_width(SCROLLBAR_WIDTH)
        .margin(0)
}

/// Creates a vertical scrollable region using Moonveil's concrete theme and
/// default scrollbar geometry.
pub fn scrollable<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Scrollable<'a, Message, Renderer>
where
    Renderer: iced_core::text::Renderer,
{
    Scrollable::with_direction(content, Direction::Vertical(scrollbar()))
}

/// A built-in Moonveil scrollable style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// A visible track and thumb for primary scrolling regions.
    #[default]
    Standard,
    /// A quiet thumb without a persistent track fill.
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

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        match class {
            Class::Variant(variant) => appearance(self, status, *variant),
            Class::Custom(style) => style(self, status),
        }
    }
}

/// Returns the resolved style for a built-in scrollable variant and status.
pub fn appearance(theme: &Theme, status: Status, variant: Variant) -> Style {
    let (horizontal_emphasis, vertical_emphasis, horizontal_disabled, vertical_disabled) =
        match status {
            Status::Active {
                is_horizontal_scrollbar_disabled,
                is_vertical_scrollbar_disabled,
            } => (
                false,
                false,
                is_horizontal_scrollbar_disabled,
                is_vertical_scrollbar_disabled,
            ),
            Status::Hovered {
                is_horizontal_scrollbar_hovered,
                is_vertical_scrollbar_hovered,
                is_horizontal_scrollbar_disabled,
                is_vertical_scrollbar_disabled,
            } => (
                is_horizontal_scrollbar_hovered,
                is_vertical_scrollbar_hovered,
                is_horizontal_scrollbar_disabled,
                is_vertical_scrollbar_disabled,
            ),
            Status::Dragged {
                is_horizontal_scrollbar_dragged,
                is_vertical_scrollbar_dragged,
                is_horizontal_scrollbar_disabled,
                is_vertical_scrollbar_disabled,
            } => (
                is_horizontal_scrollbar_dragged,
                is_vertical_scrollbar_dragged,
                is_horizontal_scrollbar_disabled,
                is_vertical_scrollbar_disabled,
            ),
        };

    Style {
        container: iced_widget::container::Style::default(),
        vertical_rail: rail(theme, variant, vertical_emphasis, vertical_disabled),
        horizontal_rail: rail(theme, variant, horizontal_emphasis, horizontal_disabled),
        gap: None,
        auto_scroll: AutoScroll {
            background: Background::Color(theme.surface(Surface::Overlay).into()),
            border: Border {
                color: theme.colors().borders.standard.into(),
                width: theme.appearance().border.control,
                radius: theme.appearance().radius.full.into(),
            },
            shadow: theme.appearance().shadow.popover.into(),
            icon: theme.colors().content.secondary.into(),
        },
    }
}

fn rail(theme: &Theme, variant: Variant, emphasized: bool, disabled: bool) -> Rail {
    let track_border = match variant {
        Variant::Standard => Border {
            color: theme.colors().borders.subtle.into(),
            width: theme.appearance().border.hairline,
            radius: theme.appearance().radius.full.into(),
        },
        Variant::Subtle => Border::default(),
    };
    let thumb = if disabled {
        theme.colors().content.disabled
    } else if emphasized {
        theme.colors().content.primary
    } else {
        theme.colors().content.secondary
    };

    Rail {
        background: match variant {
            Variant::Standard => Some(Background::Color(theme.surface(Surface::Inset).into())),
            Variant::Subtle => None,
        },
        border: track_border,
        scroller: Scroller {
            background: Background::Color(thumb.into()),
            border: Border {
                color: theme.colors().borders.standard.into(),
                width: theme.appearance().border.hairline,
                radius: theme.appearance().radius.full.into(),
            },
        },
    }
}
