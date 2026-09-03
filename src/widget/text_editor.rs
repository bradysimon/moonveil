//! Text editors using Moonveil's concrete theme and resolved field tokens.

use crate::{Theme, widget::field};

pub use iced_core::widget::Id;
pub use iced_widget::text_editor::{
    Action, Binding, Catalog, Content, Cursor, Edit, KeyPress, Line, LineEnding, Motion, Selection,
    Status, Style,
};

/// A custom text editor style function using Moonveil's concrete theme.
pub type StyleFn<'a> = iced_widget::text_editor::StyleFn<'a, Theme>;

/// An Iced plain-text editor using Moonveil's concrete theme.
pub type TextEditor<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::TextEditor<'a, iced_core::text::highlighter::PlainText, Message, Theme, Renderer>;

/// Creates a plain-text editor using Moonveil's concrete theme.
pub fn text_editor<'a, Message, Renderer>(
    content: &'a Content<Renderer>,
) -> TextEditor<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::text::Renderer,
{
    TextEditor::new(content)
}

/// A built-in Moonveil text editor style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// An editable field with a strong boundary.
    #[default]
    Standard,
    /// A lower-emphasis editable field.
    Subtle,
    /// A recessed field intended for code-like content.
    Code,
    /// A field whose value is invalid.
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

/// Returns the resolved style for a built-in text editor variant and status.
pub fn appearance(theme: &Theme, status: Status, variant: Variant) -> Style {
    let appearance = field::appearance(theme, field_status(status), field_variant(variant));

    Style {
        background: appearance.background,
        border: appearance.border,
        placeholder: appearance.placeholder,
        value: appearance.value,
        selection: appearance.selection,
    }
}

fn field_status(status: Status) -> field::Status {
    match status {
        Status::Active => field::Status::Active,
        Status::Hovered => field::Status::Hovered,
        Status::Focused { .. } => field::Status::Focused,
        Status::Disabled => field::Status::Disabled,
    }
}

fn field_variant(variant: Variant) -> field::Variant {
    match variant {
        Variant::Standard => field::Variant::Standard,
        Variant::Subtle => field::Variant::Subtle,
        Variant::Code => field::Variant::Code,
        Variant::Invalid => field::Variant::Invalid,
    }
}
