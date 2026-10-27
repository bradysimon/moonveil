//! Searchable combo boxes using Moonveil's concrete theme and resolved field tokens.

use std::fmt::Display;

use crate::{
    Theme,
    widget::{overlay::menu, text_input},
};

pub use iced_widget::combo_box::{Catalog, State};
pub use iced_widget::text_input::{Status, Style};

pub type ComboBox<'a, T, Message> = iced_widget::ComboBox<'a, T, Message, Theme>;

/// Creates a new [`ComboBox`] with the given `state`, a placeholder,
/// the current selected value, and the message to produce when an option is
/// selected.
pub fn combo_box<'a, T, Message>(
    state: &'a State<T>,
    placeholder: impl iced_core::text::IntoFragment<'a>,
    selection: Option<&T>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> ComboBox<'a, T, Message>
where
    T: Display + Clone,
{
    ComboBox::new(state, placeholder, selection, on_selected)
}

/// A built-in Moonveil combo box style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// A searchable choice field with a strong boundary.
    #[default]
    Standard,
    /// A lower-emphasis searchable choice field.
    Subtle,
    /// A searchable choice field whose value is invalid.
    Invalid,
}

/// Adds Moonveil's named variants to Iced combo boxes.
pub trait ComboBoxExt: Sized {
    /// Sets a built-in Moonveil variant for both the input and its menu.
    fn class(self, variant: Variant) -> Self;
}

impl<T, Message> ComboBoxExt for ComboBox<'_, T, Message>
where
    T: Display + Clone,
{
    fn class(self, variant: Variant) -> Self {
        self.input_class(input_variant(variant))
            .menu_class(menu_variant(variant))
    }
}

impl Catalog for Theme {
    fn default_input<'a>() -> <Self as text_input::Catalog>::Class<'a> {
        input_variant(Variant::default()).into()
    }

    fn default_menu<'a>() -> <Self as menu::Catalog>::Class<'a> {
        menu_variant(Variant::default()).into()
    }
}

/// Returns the resolved input style for a built-in combo box variant and status.
pub fn appearance(theme: &Theme, status: Status, variant: Variant) -> Style {
    text_input::appearance(theme, status, input_variant(variant))
}

/// Returns the resolved menu style for a built-in combo box variant.
pub fn menu_appearance(theme: &Theme, variant: Variant) -> menu::Style {
    menu::appearance(theme, menu_variant(variant))
}

fn input_variant(variant: Variant) -> text_input::Variant {
    match variant {
        Variant::Standard => text_input::Variant::Standard,
        Variant::Subtle => text_input::Variant::Subtle,
        Variant::Invalid => text_input::Variant::Invalid,
    }
}

fn menu_variant(variant: Variant) -> menu::Variant {
    match variant {
        Variant::Standard | Variant::Invalid => menu::Variant::Standard,
        Variant::Subtle => menu::Variant::Subtle,
    }
}
