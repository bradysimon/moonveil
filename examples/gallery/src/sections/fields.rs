use crate::gallery::Gallery;
use crate::message::Message;
use crate::shared::Field;
use crate::ui::{band, matrix_label, page, panel, section_heading};
use iced::Length::Fill;
use moonveil::widget::{column, row};
use iced::{Alignment, Length};
use moonveil::widget::text::TextExt as _;
use moonveil::widget::{Labeled, labeled, text, text_editor, text_input};
use moonveil::{Element, spacing};

pub(super) fn fields(gallery: &Gallery) -> Element<'_, Message> {
    page([
        panel(section_heading("FIELD CATALOGS", "Structured input", None)).into(),
        band([live_fields(gallery), live_editor(gallery)]),
        field_state_matrix(gallery),
    ])
}

fn live_fields(gallery: &Gallery) -> Element<'_, Message> {
    panel(
        column![
            text("Request details").class(text::Variant::Primary),
            labeled_input(
                "Request name",
                "Name this request",
                &gallery.request_name,
                Field::RequestName,
                text_input::Variant::Standard,
            )
            .required(true)
            .trailing(
                text(format!("{} / 60", gallery.request_name.chars().count()))
                    .size(text::Size::Label)
                    .class(text::Variant::Muted),
            ),
            labeled_input(
                "Search",
                "Filter headers and variables",
                &gallery.search_query,
                Field::Search,
                text_input::Variant::Subtle,
            ),
            labeled_input(
                "Callback URL",
                "https://example.com/hook",
                &gallery.endpoint,
                Field::Endpoint,
                text_input::Variant::Invalid,
            )
            .required(true)
            .error("Enter an HTTPS callback URL"),
            labeled_input(
                "Environment value",
                "KEY=value",
                &gallery.code_value,
                Field::CodeValue,
                text_input::Variant::Code,
            ),
        ]
        .spacing(spacing::SM),
    )
    .width(Length::FillPortion(3))
    .into()
}

fn labeled_input<'a>(
    label: &'static str,
    placeholder: &'static str,
    value: &'a str,
    field: Field,
    variant: text_input::Variant,
) -> Labeled<'a, Message> {
    labeled(
        label,
        text_input(placeholder, value)
            .on_input(move |value| Message::EditField(field, value))
            .class(variant)
            .padding([spacing::SM, 10.0]),
    )
}

fn live_editor(gallery: &Gallery) -> Element<'_, Message> {
    panel(
        column![
            text("Deployment notes").class(text::Variant::Primary),
            text("Plain text editor with code-oriented field treatment")
                .size(text::Size::Label)
                .class(text::Variant::Muted),
            text_editor(&gallery.notes)
                .placeholder("Add rollout notes...")
                .on_action(Message::EditNotes)
                .class(text_editor::Variant::Code)
                .font(iced::Font::MONOSPACE)
                .height(172)
                .padding(spacing::MD),
            text(format!("{} lines", gallery.notes.line_count()))
                .size(text::Size::Label)
                .class(text::Variant::Muted),
        ]
        .spacing(9),
    )
    .width(Length::FillPortion(2))
    .into()
}

fn field_state_matrix(gallery: &Gallery) -> Element<'_, Message> {
    panel(
        column![
            row![
                text("STATE")
                    .size(text::Size::Label.sm())
                    .class(text::Variant::Muted)
                    .width(96),
                matrix_label("TEXT INPUT"),
                matrix_label("TEXT EDITOR"),
            ]
            .spacing(10),
            field_state_row(gallery, "Active", text_input::Status::Active),
            field_state_row(gallery, "Hovered", text_input::Status::Hovered),
            field_state_row(
                gallery,
                "Focused",
                text_input::Status::Focused { is_hovered: false },
            ),
            field_state_row(
                gallery,
                "Focus + hover",
                text_input::Status::Focused { is_hovered: true },
            ),
            field_state_row(gallery, "Disabled", text_input::Status::Disabled),
        ]
        .spacing(spacing::SM),
    )
    .into()
}

fn field_state_row<'a>(
    gallery: &'a Gallery,
    label: &'static str,
    status: text_input::Status,
) -> Element<'a, Message> {
    let editor_status = match status {
        text_input::Status::Active => text_editor::Status::Active,
        text_input::Status::Hovered => text_editor::Status::Hovered,
        text_input::Status::Focused { is_hovered } => text_editor::Status::Focused { is_hovered },
        text_input::Status::Disabled => text_editor::Status::Disabled,
    };

    row![
        text(label)
            .size(text::Size::Label)
            .class(text::Variant::Muted)
            .width(96),
        text_input("Placeholder", "Example value")
            .on_input(|_| Message::Action)
            .style(move |theme, _| {
                text_input::appearance(theme, status, text_input::Variant::Standard)
            })
            .padding([6, 9])
            .width(Fill),
        text_editor(&gallery.notes)
            .on_action(Message::EditNotes)
            .style(move |theme, _| {
                text_editor::appearance(theme, editor_status, text_editor::Variant::Standard)
            })
            .height(34)
            .padding([6, 9]),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}
