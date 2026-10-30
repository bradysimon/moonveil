//! Alerts are inline messages carrying semantic intent.

use crate::{
    Element, Theme, icon, spacing,
    token::Intent,
    widget::{
        button, column, container, row,
        svg::{self, svg},
        text::{self, IntoFragment, LineHeight, Text},
    },
};
use iced_core::{Alignment, Length, Pixels};

const ICON_SIZE: f32 = 16.0;
const DISMISS_ICON_SIZE: f32 = 14.0;

/// Creates an informational alert displaying `title`.
pub fn alert<'a, Message>(title: impl IntoFragment<'a>) -> Alert<'a, Message> {
    Alert::new(title)
}

/// An inline message describing the state of nearby content.
pub struct Alert<'a, Message> {
    intent: Intent,
    title: Text<'a>,
    description: Option<Text<'a>>,
    on_dismiss: Option<Message>,
    width: Length,
}

impl<'a, Message> Alert<'a, Message> {
    /// Creates an informational alert displaying `title`.
    pub fn new(title: impl IntoFragment<'a>) -> Self {
        Self {
            intent: Intent::Accent,
            title: text::text(title),
            description: None,
            on_dismiss: None,
            width: Length::Fill,
        }
    }

    /// Sets the semantic intent of the alert, [`Intent::Accent`] by default.
    #[must_use]
    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    /// Sets the supporting text shown below the title.
    #[must_use]
    pub fn description(mut self, description: impl IntoFragment<'a>) -> Self {
        self.description = Some(text::text(description).size(text::size::LABEL));
        self
    }

    /// Shows a dismiss button that produces the given message when pressed.
    #[must_use]
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }

    /// Sets the width of the alert.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, Message, Renderer> From<Alert<'a, Message>> for Element<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::text::Renderer + iced_core::svg::Renderer + 'a,
{
    fn from(alert: Alert<'a, Message>) -> Self {
        let intent = alert.intent;
        let handle = match intent {
            Intent::Accent => icon::info(),
            Intent::Success => icon::check(),
            Intent::Warning => icon::triangle_alert(),
            Intent::Danger => icon::circle_alert(),
        };
        // Center icons on the title's first line rather than the whole message.
        let line_height = LineHeight::default().to_absolute(Pixels(text::size::BODY));

        let icon = container(
            svg(handle)
                .width(ICON_SIZE)
                .height(ICON_SIZE)
                .style(move |theme, _status| foreground(theme, intent)),
        )
        .center_y(line_height);

        let message = column([alert.title.into()])
            .push(alert.description)
            .spacing(spacing::XXS)
            .width(Length::Fill);

        let mut content = row([icon.into(), message.into()])
            .spacing(spacing::SM)
            .align_y(Alignment::Start);
        if let Some(on_dismiss) = alert.on_dismiss {
            content = content.push(
                button(
                    svg(icon::close())
                        .width(DISMISS_ICON_SIZE)
                        .height(DISMISS_ICON_SIZE)
                        .style(move |theme, _status| foreground(theme, intent)),
                )
                .class(button::Variant::Ghost)
                .padding((line_height.0 - DISMISS_ICON_SIZE) / 2.0)
                .on_press(on_dismiss),
            );
        }

        container(content)
            .class(container::Variant::Semantic {
                intent,
                style: container::SemanticStyle::Soft,
            })
            .padding([spacing::SM + spacing::XXS, spacing::MD])
            .width(alert.width)
            .into()
    }
}

/// Icons share the soft fill's text color.
fn foreground(theme: &Theme, intent: Intent) -> svg::Style {
    svg::Style {
        color: Some(theme.colors().semantic(intent).soft.active.text.into()),
    }
}
