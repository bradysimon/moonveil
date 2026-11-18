use crate::message::Message;
use crate::ui::{matrix_label, page, panel, section_heading};
use iced::Alignment;
use iced::Length::Fill;
use moonveil::Widget;
use moonveil::widget::text::TextExt as _;
use moonveil::widget::{button, container, text};
use moonveil::widget::{column, row};
use moonveil::{spacing, token::Intent};

pub(super) fn actions() -> impl Widget<Message> + 'static {
    page([
        panel(section_heading(
            "BUTTON CATALOG",
            "Actions and explicit states",
            None,
        ))
        .boxed(),
        panel(live_actions()).boxed(),
        panel(
            column![
                matrix_heading(),
                variant_row("Primary", button::Variant::Primary),
                variant_row("Secondary", button::Variant::Secondary),
                variant_row("Outline", button::Variant::Outline),
                variant_row("Ghost", button::Variant::Ghost),
                variant_row(
                    "Success solid",
                    button::Variant::Semantic {
                        intent: Intent::Success,
                        style: button::SemanticStyle::Solid,
                    },
                ),
                variant_row(
                    "Danger soft",
                    button::Variant::Semantic {
                        intent: Intent::Danger,
                        style: button::SemanticStyle::Soft,
                    },
                ),
                variant_row(
                    "Warning outline",
                    button::Variant::Semantic {
                        intent: Intent::Warning,
                        style: button::SemanticStyle::Outline,
                    },
                ),
            ]
            .spacing(spacing::SM),
        )
        .boxed(),
    ])
}

fn live_actions() -> impl Widget<Message> + 'static {
    container(
        column![
            row![
                button(text("Primary"))
                    .class(button::Variant::Primary)
                    .on_press(Message::Action),
                button(text("Secondary"))
                    .class(button::Variant::Secondary)
                    .on_press(Message::Action),
                button(text("Outline"))
                    .class(button::Variant::Outline)
                    .on_press(Message::Action),
                button(text("Ghost"))
                    .class(button::Variant::Ghost)
                    .on_press(Message::Action),
                button(text("Disabled")).class(button::Variant::Secondary),
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center),
            row![
                button(text("Success (solid)"))
                    .class(button::Variant::Semantic {
                        intent: Intent::Success,
                        style: button::SemanticStyle::Solid,
                    })
                    .on_press(Message::Action),
                button(text("Danger (soft)"))
                    .class(button::Variant::Semantic {
                        intent: Intent::Danger,
                        style: button::SemanticStyle::Soft,
                    })
                    .on_press(Message::Action),
                button(text("Warning (outline)"))
                    .class(button::Variant::Semantic {
                        intent: Intent::Warning,
                        style: button::SemanticStyle::Outline,
                    })
                    .on_press(Message::Action),
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center),
        ]
        .spacing(spacing::SM),
    )
    .class(container::Variant::Transparent)
    .width(Fill)
}

fn matrix_heading() -> impl Widget<Message> + 'static {
    row![
        text("VARIANT")
            .size(text::Size::Label)
            .class(text::Variant::Muted)
            .width(108),
        matrix_label("ACTIVE"),
        matrix_label("HOVERED"),
        matrix_label("PRESSED"),
        matrix_label("DISABLED"),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
}

fn variant_row(label: &'static str, variant: button::Variant) -> impl Widget<Message> + 'static {
    row![
        text(label)
            .size(text::Size::Label)
            .class(text::Variant::Secondary)
            .width(108),
        forced_button("Active", variant, button::Status::Active),
        forced_button("Hovered", variant, button::Status::Hovered),
        forced_button("Pressed", variant, button::Status::Pressed),
        forced_button("Disabled", variant, button::Status::Disabled),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
}

fn forced_button(
    label: &'static str,
    variant: button::Variant,
    status: button::Status,
) -> impl Widget<Message> + 'static {
    button(text(label).size(text::Size::Label))
        .style(move |theme, _runtime_status| button::appearance(theme, status, variant))
        .padding([6, 10])
        .width(Fill)
}
