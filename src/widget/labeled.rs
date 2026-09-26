//! A caption placed above a control, with optional supporting text below it.

use std::borrow::Cow;

use crate::{
    Element, spacing,
    token::Intent,
    widget::{
        Column, Row, Space,
        text::{self, IntoFragment, LineHeight, Text, TextExt as _, Variant},
    },
};
use iced_core::{Alignment, Length, Pixels};

/// Supporting text that may be absent, such as a conditional validation message.
pub trait IntoMessage<'a> {
    /// Converts into the text to display, if any.
    fn into_message(self) -> Option<Cow<'a, str>>;
}

impl<'a> IntoMessage<'a> for &'a str {
    fn into_message(self) -> Option<Cow<'a, str>> {
        Some(Cow::Borrowed(self))
    }
}

impl<'a> IntoMessage<'a> for &'a String {
    fn into_message(self) -> Option<Cow<'a, str>> {
        Some(Cow::Borrowed(self))
    }
}

impl<'a> IntoMessage<'a> for String {
    fn into_message(self) -> Option<Cow<'a, str>> {
        Some(Cow::Owned(self))
    }
}

impl<'a> IntoMessage<'a> for Cow<'a, str> {
    fn into_message(self) -> Option<Cow<'a, str>> {
        Some(self)
    }
}

impl<'a, T: IntoMessage<'a>> IntoMessage<'a> for Option<T> {
    fn into_message(self) -> Option<Cow<'a, str>> {
        self.and_then(IntoMessage::into_message)
    }
}

/// A control paired with a caption and optional supporting text.
///
/// Space for one line of supporting text is reserved by default so messages
/// can appear without shifting the surrounding layout. You can disable this
/// behavior with [`Labeled::reserve_message`].
pub struct Labeled<'a, Message, Renderer = iced_widget::Renderer>
where
    Renderer: iced_core::text::Renderer,
{
    label: Text<'a>,
    content: Element<'a, Message, Renderer>,
    required: bool,
    trailing: Option<Element<'a, Message, Renderer>>,
    hint: Option<Text<'a>>,
    message: Option<Text<'a>>,
    reserve_message: bool,
}

/// Places a caption above `content`, such as a field or pick list.
pub fn labeled<'a, Message, Renderer>(
    label: impl IntoFragment<'a>,
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Labeled<'a, Message, Renderer>
where
    Renderer: iced_core::text::Renderer,
{
    Labeled {
        label: text::text(label)
            .size(text::size::LABEL)
            .class(Variant::Secondary),
        content: content.into(),
        required: false,
        trailing: None,
        hint: None,
        message: None,
        reserve_message: true,
    }
}

impl<'a, Message, Renderer> Labeled<'a, Message, Renderer>
where
    Renderer: iced_core::text::Renderer,
{
    /// Sets whether the label is marked as required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Places compact content at the end of the label row, such as a character
    /// count, a reset action, or an info icon.
    pub fn trailing(mut self, trailing: impl Into<Element<'a, Message, Renderer>>) -> Self {
        self.trailing = Some(trailing.into());
        self
    }

    /// Adds guidance below the content, shown whenever there is no message.
    pub fn hint(mut self, hint: impl IntoMessage<'a>) -> Self {
        self.hint = hint
            .into_message()
            .map(|hint| supporting(hint, Variant::Muted));
        self
    }

    /// Shows a message carrying `intent` below the content in place of the hint.
    ///
    /// Passing `None` clears the message, which suits conditional validation.
    pub fn message(mut self, intent: Intent, message: impl IntoMessage<'a>) -> Self {
        self.message = message
            .into_message()
            .map(|message| supporting(message, Variant::Semantic(intent)));
        self
    }

    /// Shorthand for a [`Intent::Danger`] message below the content in place of the hint.
    pub fn error(self, error: impl IntoMessage<'a>) -> Self {
        self.message(Intent::Danger, error)
    }

    /// Sets whether an empty line is kept below the content when there is no
    /// hint or message.
    ///
    /// Disable this for fields that never show supporting text.
    pub fn reserve_message(mut self, reserve: bool) -> Self {
        self.reserve_message = reserve;
        self
    }
}

fn supporting<'a>(content: impl IntoFragment<'a>, variant: Variant) -> Text<'a> {
    text::text(content).size(text::size::CAPTION).class(variant)
}

impl<'a, Message, Renderer> From<Labeled<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_core::text::Renderer + 'a,
{
    fn from(labeled: Labeled<'a, Message, Renderer>) -> Self {
        let supporting: Option<Element<'a, Message, Renderer>> =
            match labeled.message.or(labeled.hint) {
                Some(text) => Some(text.into()),
                None if labeled.reserve_message => Some(
                    Space::new()
                        .height(LineHeight::default().to_absolute(Pixels(text::size::CAPTION)))
                        .into(),
                ),
                None => None,
            };
        let marker = labeled.required.then(|| {
            text::text("*")
                .size(text::size::LABEL)
                .class(Variant::Semantic(Intent::Danger))
        });
        let mut header = Row::new()
            .push(labeled.label)
            .push(marker)
            .spacing(spacing::XXS)
            .align_y(Alignment::Center);

        if let Some(trailing) = labeled.trailing {
            header = header
                .push(Space::new().width(Length::Fill))
                .push(trailing)
                .width(Length::Fill);
        }

        // Follow the control's width so a fluid header can't widen fixed-width controls.
        let width = labeled.content.as_widget().size().width;

        Column::new()
            .push(header)
            .push(labeled.content)
            .push(supporting)
            .spacing(spacing::XS)
            .width(width)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field() -> Labeled<'static, ()> {
        labeled("Name", Space::new())
    }

    #[test]
    fn reserves_message_space_by_default() {
        assert!(field().reserve_message);
        assert!(!field().reserve_message(false).reserve_message);
    }

    #[test]
    fn message_takes_precedence_over_hint_in_any_order() {
        let before = field().error("Required").hint("Shown publicly");
        let after = field().hint("Shown publicly").error("Required");

        for labeled in [before, after] {
            assert!(labeled.message.is_some());
            assert!(labeled.hint.is_some());
        }
    }

    #[test]
    fn none_clears_message() {
        let error: Option<String> = None;
        let labeled = field().error("Required").error(error);

        assert!(labeled.message.is_none());
    }

    #[test]
    fn accepts_borrowed_and_optional_references() {
        let owned = String::from("Invalid URL");
        let error: Option<&String> = Some(&owned);
        let field = || labeled::<(), iced_widget::Renderer>("Name", Space::new());

        assert!(field().error(error).message.is_some());
        assert!(field().error(&owned).message.is_some());
    }

    #[test]
    fn accepts_owned_and_conditional_messages() {
        let labeled = field()
            .message(Intent::Warning, String::from("Overwrites the file"))
            .hint(true.then_some("Optional"));

        assert!(labeled.message.is_some());
        assert!(labeled.hint.is_some());
    }

    fn layout_width(content: Space) -> f32 {
        use iced_core::{Size, layout::Limits, widget::Tree};

        let mut element: Element<'_, (), ()> = labeled("Name", content)
            .required(true)
            .trailing(text::text("0 / 60"))
            .into();
        let mut tree = Tree::empty();
        tree.diff(element.as_widget_mut());
        let limits = Limits::new(Size::ZERO, Size::new(800.0, 600.0));

        element.as_widget_mut().layout(&mut tree, &(), &limits);
        tree.size.width
    }

    #[test]
    fn trailing_content_does_not_widen_fixed_controls() {
        assert_eq!(layout_width(Space::new().width(140)), 140.0);
    }

    #[test]
    fn trailing_content_follows_fluid_controls() {
        assert_eq!(layout_width(Space::new().width(Length::Fill)), 800.0);
    }
}
