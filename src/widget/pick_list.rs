//! Pick lists using Moonveil's concrete theme and resolved field tokens.

use std::borrow::Borrow;

use crate::{
    Theme,
    token::{Interaction, Surface},
    widget::overlay::menu,
};
use iced_core::{Background, Border};

pub use iced_widget::pick_list::{Catalog, Handle, Status, Style};

/// A custom pick list style function using Moonveil's concrete theme.
pub type StyleFn<'a> = iced_widget::pick_list::StyleFn<'a, Theme>;

/// An Iced pick list using Moonveil's concrete theme.
pub type PickList<'a, Item, Options, Selected, Message, Renderer = iced_widget::Renderer> =
    iced_widget::PickList<'a, Item, Options, Selected, Message, Theme, Renderer>;

/// Creates a pick list using Moonveil's concrete theme.
pub fn pick_list<'a, Item, Options, Selected, Message, Renderer>(
    selected: Option<Selected>,
    options: Options,
    to_string: impl Fn(&Item) -> String + 'a,
) -> PickList<'a, Item, Options, Selected, Message, Renderer>
where
    Item: PartialEq + Clone + 'a,
    Options: Borrow<[Item]> + 'a,
    Selected: Borrow<Item> + 'a,
    Message: Clone,
    Renderer: iced_core::text::Renderer,
{
    PickList::new(selected, options, to_string)
}

/// A built-in Moonveil pick list style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// An ordinary choice field with a strong boundary.
    #[default]
    Standard,
    /// A lower-emphasis choice field.
    Subtle,
    /// A choice field whose value is invalid.
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

    fn default<'a>() -> <Self as Catalog>::Class<'a> {
        Class::default()
    }

    fn default_menu<'a>() -> <Self as iced_widget::overlay::menu::Catalog>::Class<'a> {
        menu::Variant::Standard.into()
    }

    fn style(&self, class: &<Self as Catalog>::Class<'_>, status: Status) -> Style {
        match class {
            Class::Variant(variant) => appearance(self, status, *variant),
            Class::Custom(style) => style(self, status),
        }
    }
}

/// Returns the resolved style for a built-in pick list variant and status.
pub fn appearance(theme: &Theme, status: Status, variant: Variant) -> Style {
    let disabled = status == Status::Disabled;
    let background = match status {
        Status::Hovered | Status::Opened { is_hovered: true } => {
            theme.interaction_on(Surface::Field, Interaction::Hover)
        }
        Status::Active | Status::Opened { is_hovered: false } => theme.surface(Surface::Field),
        Status::Disabled => theme.surface(Surface::Inset),
    };
    let (border_color, border_width) = match (variant, status) {
        (Variant::Invalid, Status::Opened { .. }) => (
            theme.colors().danger.border,
            theme.appearance().border.focus,
        ),
        (Variant::Invalid, _) => (
            theme.colors().danger.border,
            theme.appearance().border.control,
        ),
        (_, Status::Opened { .. }) => (
            theme.colors().borders.focus,
            theme.appearance().border.focus,
        ),
        (Variant::Subtle, _) => (
            theme.colors().borders.standard,
            theme.appearance().border.hairline,
        ),
        (Variant::Standard, _) => (
            theme.colors().borders.strong,
            theme.appearance().border.control,
        ),
    };
    let value = if disabled {
        theme.colors().content.disabled
    } else {
        theme.colors().content.primary
    };
    let secondary = if disabled {
        theme.colors().content.disabled
    } else {
        theme.colors().content.muted
    };

    Style {
        text_color: value.into(),
        placeholder_color: secondary.into(),
        handle_color: secondary.into(),
        background: Background::Color(background.into()),
        border: Border {
            color: border_color.into(),
            width: border_width,
            radius: theme.appearance().radius.sm.into(),
        },
    }
}
