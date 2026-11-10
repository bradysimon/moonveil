use crate::message::Message;
use crate::ui::{band, matrix_label, page, palette_heading, panel, section_heading, PANEL_PADDING};
use iced::Alignment;
use iced::Length::Fill;
use moonveil::widget::{column, row};
use moonveil::widget::text::TextExt as _;
use moonveil::widget::{container, text};
use moonveil::{Element, spacing, token::Intent};

pub(super) fn surfaces() -> Element<'static, Message> {
    page([
        panel(section_heading(
            "CONTENT FOUNDATIONS",
            "Planes and readable roles",
            None,
        ))
        .into(),
        panel(palette_heading("FLUSH", "Background surfaces")).into(),
        band([
            plane_swatch("Canvas", "App chrome", container::Variant::Canvas),
            plane_swatch("Surface", "Primary plane", container::Variant::Surface),
            plane_swatch("Sunken", "Wells and output", container::Variant::Sunken),
            plane_swatch("Raised", "Toolbars and bands", container::Variant::Raised),
        ]),
        panel(
            column![
                palette_heading("FLOATING", "Content and field surfaces"),
                row![
                    surface_swatch(
                        "Raised",
                        ".floating()",
                        container::Variant::Raised.floating(),
                    ),
                    surface_swatch(
                        "Sunken",
                        ".floating()",
                        container::Variant::Sunken.floating(),
                    ),
                    surface_swatch("Inset", "Local recess", container::Variant::Inset),
                    surface_swatch("Field", "Editable plane", container::Variant::Field),
                    surface_swatch("Overlay", "Placed above", container::Variant::Overlay),
                    surface_swatch("Popover", "Anchored", container::Variant::Popover),
                ]
                .spacing(spacing::MD),
            ]
            .spacing(spacing::MD),
        )
        .into(),
        panel(
            row![
                content_sample("Primary", text::Variant::Primary),
                content_sample("Secondary", text::Variant::Secondary),
                content_sample("Muted", text::Variant::Muted),
                content_sample("Decorative", text::Variant::Decorative),
                content_sample("Disabled", text::Variant::Disabled),
            ]
            .spacing(spacing::LG),
        )
        .into(),
        panel(
            column![
                row![
                    text("ROLE")
                        .size(text::Size::Label)
                        .class(text::Variant::Muted)
                        .width(100),
                    matrix_label("SMALL"),
                    matrix_label("MEDIUM"),
                    matrix_label("LARGE"),
                ]
                .spacing(spacing::MD),
                type_scale_row("Label", text::Size::Label),
                type_scale_row("Body", text::Size::Body),
                type_scale_row("Title", text::Size::Title),
                type_scale_row("Headline", text::Size::Headline),
                type_scale_row("Display", text::Size::Display),
            ]
            .spacing(spacing::MD),
        )
        .into(),
        panel(
            row![
                semantic_sample("Success", Intent::Success),
                semantic_sample("Warning", Intent::Warning),
                semantic_sample("Danger", Intent::Danger),
            ]
            .spacing(spacing::MD),
        )
        .into(),
    ])
}

fn plane_swatch(
    title: &'static str,
    detail: &'static str,
    variant: container::Variant,
) -> Element<'static, Message> {
    container(
        column![
            text(title).class(text::Variant::Primary),
            text(detail)
                .size(text::Size::Label)
                .class(text::Variant::Muted),
        ]
        .spacing(6),
    )
    .class(variant)
    .padding(PANEL_PADDING)
    .width(Fill)
    .height(112)
    .into()
}

fn surface_swatch(
    title: &'static str,
    detail: &'static str,
    class: impl Into<container::Class<'static>>,
) -> Element<'static, Message> {
    container(
        column![
            text(title).class(text::Variant::Primary),
            text(detail)
                .size(text::Size::Label)
                .class(text::Variant::Muted),
        ]
        .spacing(6),
    )
    .class(class)
    .padding(14)
    .width(Fill)
    .height(92)
    .into()
}

fn content_sample(label: &'static str, variant: text::Variant) -> Element<'static, Message> {
    column![
        text("Aa").size(text::Size::Headline).class(variant),
        text(label).size(text::Size::Label).class(variant),
    ]
    .spacing(3)
    .width(Fill)
    .into()
}

fn type_scale_row(label: &'static str, size: text::Size) -> Element<'static, Message> {
    row![
        text(label)
            .size(text::Size::Label)
            .class(text::Variant::Secondary)
            .width(100),
        type_scale_sample(size, text::Scale::Small),
        type_scale_sample(size, text::Scale::Medium),
        type_scale_sample(size, text::Scale::Large),
    ]
    .spacing(spacing::MD)
    .align_y(Alignment::Center)
    .into()
}

fn type_scale_sample(size: text::Size, scale: text::Scale) -> Element<'static, Message> {
    let pixels = size.px(scale);
    column![
        text("Aa").size(pixels).class(text::Variant::Primary),
        text(format!("{:.0} px", pixels.0))
            .size(text::Size::Label)
            .class(text::Variant::Muted),
    ]
    .spacing(spacing::XS)
    .align_x(Alignment::Center)
    .width(Fill)
    .into()
}

fn semantic_sample(label: &'static str, intent: Intent) -> Element<'static, Message> {
    container(column![text(label), text("Soft pair").size(text::Size::Label),].spacing(spacing::XS))
        .class(container::Variant::Semantic {
            intent,
            style: container::SemanticStyle::Soft,
        })
        .padding(14)
        .width(Fill)
        .into()
}
