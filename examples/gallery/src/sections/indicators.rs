use crate::message::Message;
use crate::ui::{band, matrix_label, page, panel, section_heading};
use iced::Alignment;
use iced::Length::Fill;
use moonveil::widget::{column, row};
use moonveil::widget::text::TextExt as _;
use moonveil::widget::{alert, badge, button, container, progress, progress_bar, rule, text, Space};
use moonveil::{Element, spacing, token::Intent};

pub(super) fn indicators() -> Element<'static, Message> {
    page([
        panel(section_heading(
            "CONTENT CATALOGS",
            "Badges, alerts, separation, and progress",
            None,
        ))
        .into(),
        band([badge_specimens(), badge_matrix()]),
        alert_specimens(),
        band([operation_progress(), quota_progress()]),
        progress_matrix(),
        indeterminate_matrix(),
        band([indeterminate_in_context(), indeterminate_sizes()]),
        rule_specimens(),
    ])
}

const COMPACT_SHAPES: [(&str, progress::Shape); 7] = [
    ("ARC", progress::Shape::Arc),
    ("SWEEP", progress::Shape::Sweep),
    ("SPOKES", progress::Shape::Spokes),
    ("ORBIT", progress::Shape::Orbit),
    ("DOTS", progress::Shape::Dots),
    ("BARS", progress::Shape::Bars),
    ("PULSE", progress::Shape::Pulse),
];

fn indeterminate_matrix() -> Element<'static, Message> {
    let variant_row = |label: &'static str, variant: progress::Variant| {
        let mut cells = row![
            text(label)
                .size(text::Size::Label)
                .class(text::Variant::Secondary)
                .width(84),
        ]
        .spacing(spacing::MD)
        .align_y(Alignment::Center);

        for (_, shape) in COMPACT_SHAPES {
            cells = cells.push(container(progress().shape(shape).class(variant)).center_x(Fill));
        }

        column![
            cells,
            row![
                Space::new().width(84),
                progress().shape(progress::Shape::Bar).class(variant),
            ]
            .spacing(spacing::MD)
            .align_y(Alignment::Center),
        ]
        .spacing(spacing::SM)
    };

    let mut heading = row![
        text("VARIANT")
            .size(text::Size::Label)
            .class(text::Variant::Muted)
            .width(84),
    ]
    .spacing(spacing::MD)
    .align_y(Alignment::Center);

    for (label, _) in COMPACT_SHAPES {
        heading = heading.push(matrix_label(label));
    }

    panel(
        column![
            heading,
            variant_row("Standard", progress::Variant::Standard),
            variant_row("Neutral", progress::Variant::Neutral),
            variant_row("Success", progress::Variant::Semantic(Intent::Success)),
            variant_row("Danger", progress::Variant::Semantic(Intent::Danger)),
        ]
        .spacing(spacing::LG),
    )
    .into()
}

fn indeterminate_in_context() -> Element<'static, Message> {
    let small = |shape| progress().shape(shape).size(progress::Size::Small);

    panel(
        column![
            row![
                button(
                    row![
                        small(progress::Shape::Arc).class(progress::Variant::Inherited),
                        text("Saving"),
                    ]
                    .spacing(spacing::SM)
                    .align_y(Alignment::Center),
                )
                .class(button::Variant::Primary)
                .on_press(Message::Action),
                button(
                    row![
                        small(progress::Shape::Spokes).class(progress::Variant::Inherited),
                        text("Refreshing"),
                    ]
                    .spacing(spacing::SM)
                    .align_y(Alignment::Center),
                )
                .class(button::Variant::Secondary)
                .on_press(Message::Action),
            ]
            .spacing(spacing::SM),
            row![
                small(progress::Shape::Dots).class(progress::Variant::Neutral),
                text("Assistant is typing")
                    .size(text::Size::Label)
                    .class(text::Variant::Muted),
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center),
            row![
                small(progress::Shape::Pulse).class(progress::Variant::Semantic(Intent::Success)),
                text("Live: streaming logs")
                    .size(text::Size::Label)
                    .class(text::Variant::Secondary),
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center),
            column![
                text("Fetching schema")
                    .size(text::Size::Label)
                    .class(text::Variant::Secondary),
                progress()
                    .shape(progress::Shape::Bar)
                    .size(progress::Size::Small),
            ]
            .spacing(spacing::XS),
        ]
        .spacing(spacing::MD),
    )
    .into()
}

fn indeterminate_sizes() -> Element<'static, Message> {
    let sizes = |shape| {
        row![
            progress().shape(shape).size(progress::Size::Small),
            progress().shape(shape),
            progress().shape(shape).size(progress::Size::Large),
        ]
        .spacing(spacing::LG)
        .align_y(Alignment::Center)
    };

    panel(
        column![
            text("SIZES")
                .size(text::Size::Label)
                .class(text::Variant::Muted),
            sizes(progress::Shape::Arc),
            sizes(progress::Shape::Orbit),
            sizes(progress::Shape::Dots),
            sizes(progress::Shape::Pulse),
        ]
        .spacing(spacing::MD),
    )
    .into()
}

fn badge_specimens() -> Element<'static, Message> {
    let soft = |intent| badge::Variant::Semantic {
        intent,
        style: badge::SemanticStyle::Soft,
    };
    let request = |method: &'static str, intent, path: &'static str| {
        row![
            badge(method)
                .class(soft(intent))
                .size(badge::Size::Small)
                .width(52),
            text(path)
                .size(text::Size::Label)
                .class(text::Variant::Secondary),
        ]
        .spacing(spacing::SM)
        .align_y(Alignment::Center)
    };

    panel(
        column![
            row![
                text("Sessions API")
                    .size(text::Size::Title)
                    .class(text::Variant::Primary),
                badge("3").shape(badge::Shape::Pill),
                Space::new().width(Fill),
                badge("200 OK").class(soft(Intent::Success)),
                badge("142 ms").class(badge::Variant::Outline),
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center),
            rule::horizontal(1).class(rule::Variant::Subtle),
            request("GET", Intent::Accent, "/v1/sessions"),
            request("POST", Intent::Success, "/v1/sessions"),
            request("DELETE", Intent::Danger, "/v1/sessions/{id}"),
            rule::horizontal(1).class(rule::Variant::Subtle),
            row![
                badge("Active").class(badge::Variant::Semantic {
                    intent: Intent::Accent,
                    style: badge::SemanticStyle::Solid,
                }),
                badge("string").class(badge::Variant::Outline),
                badge("deprecated").class(badge::Variant::Semantic {
                    intent: Intent::Warning,
                    style: badge::SemanticStyle::Outline,
                }),
            ]
            .spacing(spacing::XS)
            .align_y(Alignment::Center),
        ]
        .spacing(spacing::MD),
    )
    .into()
}

fn badge_matrix() -> Element<'static, Message> {
    let intent_row = |label: &'static str, intent| {
        row![
            text(label)
                .size(text::Size::Label)
                .class(text::Variant::Secondary)
                .width(84),
            badge_cell(badge::Variant::Semantic {
                intent,
                style: badge::SemanticStyle::Solid,
            }),
            badge_cell(badge::Variant::Semantic {
                intent,
                style: badge::SemanticStyle::Soft,
            }),
            badge_cell(badge::Variant::Semantic {
                intent,
                style: badge::SemanticStyle::Outline,
            }),
        ]
        .spacing(spacing::MD)
        .align_y(Alignment::Center)
    };

    panel(
        column![
            row![
                text("INTENT")
                    .size(text::Size::Label)
                    .class(text::Variant::Muted)
                    .width(84),
                matrix_label("SOLID"),
                matrix_label("SOFT"),
                matrix_label("OUTLINE"),
            ]
            .spacing(spacing::MD)
            .align_y(Alignment::Center),
            row![
                text("Neutral")
                    .size(text::Size::Label)
                    .class(text::Variant::Secondary)
                    .width(84),
                container(Space::new()).width(Fill),
                badge_cell(badge::Variant::Neutral),
                badge_cell(badge::Variant::Outline),
            ]
            .spacing(spacing::MD)
            .align_y(Alignment::Center),
            intent_row("Accent", Intent::Accent),
            intent_row("Success", Intent::Success),
            intent_row("Warning", Intent::Warning),
            intent_row("Danger", Intent::Danger),
            row![
                text("Sizes")
                    .size(text::Size::Label)
                    .class(text::Variant::Secondary)
                    .width(84),
                badge("Small").size(badge::Size::Small),
                badge("Medium"),
                badge("Large").size(badge::Size::Large),
                badge("Pill").shape(badge::Shape::Pill),
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center),
        ]
        .spacing(spacing::MD),
    )
    .into()
}

fn badge_cell(variant: badge::Variant) -> Element<'static, Message> {
    container(badge("Label").class(variant))
        .center_x(Fill)
        .into()
}

fn alert_specimens() -> Element<'static, Message> {
    panel(
        column![
            alert("A new version of the CLI is available."),
            alert("Deployment complete")
                .intent(Intent::Success)
                .description("All 12 instances are healthy and serving traffic."),
            alert("Certificate expires in 7 days")
                .intent(Intent::Warning)
                .description("Renew it before October 30 to avoid interrupted requests.")
                .on_dismiss(Message::Action),
            alert("Unable to reach the database")
                .intent(Intent::Danger)
                .description("Check the connection string and network rules, then retry."),
        ]
        .spacing(spacing::SM),
    )
    .into()
}

fn operation_progress() -> Element<'static, Message> {
    panel(
        column![
            row![
                column![
                    text("Deploying preview")
                        .size(text::Size::Title)
                        .class(text::Variant::Primary),
                    text("Optimizing application bundle")
                        .size(text::Size::Label)
                        .class(text::Variant::Muted),
                ]
                .spacing(3),
                Space::new().width(Fill),
                text("68%")
                    .size(text::Size::Body)
                    .class(text::Variant::Secondary),
            ]
            .align_y(Alignment::Center),
            progress_bar(0.0..=100.0, 68.0)
                .class(progress_bar::Variant::Standard)
                .girth(10),
            rule::horizontal(1).class(rule::Variant::Subtle),
            text("3 of 4 checks complete")
                .size(text::Size::Label)
                .class(text::Variant::Semantic(Intent::Success)),
        ]
        .spacing(spacing::MD),
    )
    .into()
}

fn quota_progress() -> Element<'static, Message> {
    panel(
        column![
            row![
                column![
                    text("Monthly request quota")
                        .size(text::Size::Title)
                        .class(text::Variant::Primary),
                    text("8,420 of 10,000 requests")
                        .size(text::Size::Label)
                        .class(text::Variant::Muted),
                ]
                .spacing(3),
                Space::new().width(Fill),
                text("84%")
                    .size(text::Size::Body)
                    .class(text::Variant::Semantic(Intent::Warning)),
            ]
            .align_y(Alignment::Center),
            progress_bar(0.0..=100.0, 84.0)
                .class(progress_bar::Variant::Semantic(Intent::Warning))
                .girth(10),
            rule::horizontal(1).class(rule::Variant::Semantic(Intent::Warning)),
            text("Quota resets in 6 days")
                .size(text::Size::Label)
                .class(text::Variant::Secondary),
        ]
        .spacing(spacing::MD),
    )
    .into()
}

fn progress_matrix() -> Element<'static, Message> {
    panel(
        column![
            row![
                text("INTENT")
                    .size(text::Size::Label)
                    .class(text::Variant::Muted)
                    .width(84),
                matrix_label("EMPTY"),
                matrix_label("PARTIAL"),
                matrix_label("FULL"),
            ]
            .spacing(spacing::MD)
            .align_y(Alignment::Center),
            progress_row("Accent", progress_bar::Variant::Standard, 46.0),
            progress_row(
                "Success",
                progress_bar::Variant::Semantic(Intent::Success),
                72.0,
            ),
            progress_row(
                "Warning",
                progress_bar::Variant::Semantic(Intent::Warning),
                84.0,
            ),
            progress_row(
                "Danger",
                progress_bar::Variant::Semantic(Intent::Danger),
                63.0,
            ),
        ]
        .spacing(spacing::MD),
    )
    .into()
}

fn progress_row(
    label: &'static str,
    variant: progress_bar::Variant,
    partial: f32,
) -> Element<'static, Message> {
    row![
        text(label)
            .size(text::Size::Label)
            .class(text::Variant::Secondary)
            .width(84),
        progress_cell(0.0, variant),
        progress_cell(partial, variant),
        progress_cell(100.0, variant),
    ]
    .spacing(spacing::MD)
    .align_y(Alignment::Center)
    .into()
}

fn progress_cell(value: f32, variant: progress_bar::Variant) -> Element<'static, Message> {
    progress_bar(0.0..=100.0, value)
        .class(variant)
        .girth(8)
        .into()
}

fn rule_specimens() -> Element<'static, Message> {
    band([
        panel(
            column![
                text("Request summary").class(text::Variant::Primary),
                rule::horizontal(1).class(rule::Variant::Standard),
                text("POST /v1/sessions")
                    .size(text::Size::Label)
                    .class(text::Variant::Secondary),
                rule::horizontal(2).class(rule::Variant::Strong),
                text("Validation failed")
                    .size(text::Size::Label)
                    .class(text::Variant::Semantic(Intent::Danger)),
                rule::horizontal(2).class(rule::Variant::Semantic(Intent::Danger)),
            ]
            .spacing(10),
        )
        .into(),
        panel(
            row![
                column![
                    text("Environment")
                        .size(text::Size::Label)
                        .class(text::Variant::Muted),
                    text("Production")
                        .size(text::Size::Body)
                        .class(text::Variant::Primary),
                ]
                .spacing(spacing::XS)
                .width(Fill),
                rule::vertical(1).class(rule::Variant::Subtle),
                column![
                    text("Region")
                        .size(text::Size::Label)
                        .class(text::Variant::Muted),
                    text("us-east-1")
                        .size(text::Size::Body)
                        .class(text::Variant::Primary),
                ]
                .spacing(spacing::XS)
                .width(Fill),
            ]
            .spacing(spacing::LG),
        )
        .into(),
    ])
}
