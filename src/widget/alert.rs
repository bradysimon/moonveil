//! Alerts are inline messages carrying semantic intent.

use crate::Widget as _;
use crate::{
    Element, Theme, icon, spacing,
    token::Intent,
    widget::{
        Compose, Composed, Row, button, column,
        container::{self, Container},
        svg::{self, svg},
        text::{self, IntoFragment, LineHeight, Text},
    },
};
use iced_core::{Alignment, Length};
use std::marker::PhantomData;

const ICON_SIZE: f32 = 16.0;
const DISMISS_ICON_SIZE: f32 = 14.0;

/// Creates an informational alert displaying `title`.
pub fn alert<'a, Message, Renderer>(title: impl IntoFragment<'a>) -> Alert<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::text::Renderer + iced_core::svg::Renderer + 'a,
{
    Alert::new(title)
}

/// An inline message describing the state of nearby content.
pub type Alert<'a, Message, Renderer = iced_widget::Renderer> =
    Composed<Settings<'a, Message, Renderer>>;

/// The configuration an [`Alert`] is built from.
pub struct Settings<'a, Message, Renderer> {
    intent: Intent,
    title: Text<'a>,
    description: Option<Text<'a>>,
    on_dismiss: Option<Message>,
    width: Length,
    renderer: PhantomData<fn() -> Renderer>,
}

impl<'a, Message, Renderer> Alert<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::text::Renderer + iced_core::svg::Renderer + 'a,
{
    /// Creates an informational alert displaying `title`.
    pub fn new(title: impl IntoFragment<'a>) -> Self {
        Settings {
            intent: Intent::Accent,
            title: text::text(title),
            description: None,
            on_dismiss: None,
            width: Length::Fill,
            renderer: PhantomData,
        }
        .into()
    }

    /// Sets the semantic intent of the alert, [`Intent::Accent`] by default.
    #[must_use]
    pub fn intent(mut self, intent: Intent) -> Self {
        self.configure(|settings| settings.intent = intent);
        self
    }

    /// Sets the supporting text shown below the title.
    #[must_use]
    pub fn description(mut self, description: impl IntoFragment<'a>) -> Self {
        let description = text::text(description).size(text::Size::Label);
        self.configure(|settings| settings.description = Some(description));
        self
    }

    /// Shows a dismiss button that produces the given message when pressed.
    #[must_use]
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.configure(|settings| settings.on_dismiss = Some(message));
        self
    }

    /// Sets the width of the alert.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.configure(|settings| settings.width = width.into());
        self
    }
}

impl<'a, Message, Renderer> Compose for Settings<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::text::Renderer + iced_core::svg::Renderer + 'a,
{
    type View = Container<'a, Row<Element<'a, Message, Renderer>>>;

    fn view(self) -> Self::View {
        let intent = self.intent;
        let message = column([Some(self.title), self.description])
            .spacing(spacing::XXS)
            .width(Length::Fill);
        let dismiss = self
            .on_dismiss
            .map(|message| dismiss_button(intent, message).boxed());
        let content = Row::from_iter([intent_icon(intent).boxed(), message.boxed()])
            .extend(dismiss)
            .spacing(spacing::SM)
            .align_y(Alignment::Start);

        Container::new(content)
            .class(container::Variant::Semantic {
                intent,
                style: container::SemanticStyle::Soft,
            })
            .padding([spacing::SM + spacing::XXS, spacing::MD])
            .width(self.width)
    }
}

fn intent_icon<'a>(intent: Intent) -> container::Container<'a, svg::Svg<'a>> {
    let handle = match intent {
        Intent::Accent => icon::info(),
        Intent::Success => icon::check(),
        Intent::Warning => icon::triangle_alert(),
        Intent::Danger => icon::circle_alert(),
    };
    // Center icons on the title's first line rather than the whole message.
    let line_height = LineHeight::default().to_absolute(text::Size::Body);

    Container::new(
        svg(handle)
            .width(ICON_SIZE)
            .height(ICON_SIZE)
            .style(move |theme, _status| foreground(theme, intent)),
    )
    .center_y(line_height)
}

fn dismiss_button<'a, Message: Clone>(
    intent: Intent,
    message: Message,
) -> button::Button<'a, Message, svg::Svg<'a>> {
    let line_height = LineHeight::default().to_absolute(text::Size::Body);
    button(
        svg(icon::close())
            .width(DISMISS_ICON_SIZE)
            .height(DISMISS_ICON_SIZE)
            .style(move |theme, _status| foreground(theme, intent)),
    )
    .class(button::Variant::Ghost)
    .padding((line_height.0 - DISMISS_ICON_SIZE) / 2.0)
    .on_press(message)
}

/// Icons share the soft fill's text color.
fn foreground(theme: &Theme, intent: Intent) -> svg::Style {
    svg::Style {
        color: Some(theme.colors().semantic(intent).soft.active.text.into()),
    }
}
