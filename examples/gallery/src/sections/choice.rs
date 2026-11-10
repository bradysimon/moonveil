use crate::gallery::Gallery;
use crate::message::Message;
use crate::shared::Environment;
use crate::ui::{band, divided, page, panel, section_heading};
use iced::Length::Fill;
use moonveil::widget::{column, row};
use iced::{Alignment, Length};
use moonveil::widget::combo_box::ComboBoxExt as _;
use moonveil::widget::text::TextExt as _;
use moonveil::widget::{combo_box, container, pick_list, scrollable, text};
use moonveil::{Element, spacing};

pub(super) fn choice(gallery: &Gallery) -> Element<'_, Message> {
    page([
        panel(section_heading(
            "CHOICE & NAVIGATION",
            "Menus and bounded regions",
            None,
        ))
        .into(),
        band([pick_list_specimens(gallery), pick_list_matrix()]),
        combo_box_specimens(gallery),
        scrollable_specimens(),
    ])
}

fn pick_list_specimens(gallery: &Gallery) -> Element<'_, Message> {
    panel(
        column![
            text("Live environment selector").class(text::Variant::Primary),
            pick_list(Some(gallery.environment), Environment::ALL, |environment| {
                environment.label().to_owned()
            },)
            .on_select(Message::SelectEnvironment)
            .class(pick_list::Variant::Standard)
            .width(Fill),
            pick_list(
                Option::<Environment>::None,
                Environment::ALL,
                |environment| environment.label().to_owned(),
            )
            .placeholder("Subtle placeholder")
            .on_select(Message::SelectEnvironment)
            .class(pick_list::Variant::Subtle)
            .width(Fill),
            pick_list(
                Option::<Environment>::None,
                Environment::ALL,
                |environment| environment.label().to_owned(),
            )
            .placeholder("Selection required")
            .on_select(Message::SelectEnvironment)
            .class(pick_list::Variant::Invalid)
            .width(Fill),
        ]
        .spacing(9),
    )
    .width(Length::FillPortion(2))
    .into()
}

fn pick_list_matrix() -> Element<'static, Message> {
    panel(
        column![
            text("Explicit states")
                .size(text::Size::Title.sm())
                .class(text::Variant::Primary),
            pick_list_state("Active", pick_list::Status::Active),
            pick_list_state("Hovered", pick_list::Status::Hovered),
            pick_list_state("Opened", pick_list::Status::Opened { is_hovered: false },),
            pick_list_state(
                "Open + hover",
                pick_list::Status::Opened { is_hovered: true },
            ),
            pick_list_state("Disabled", pick_list::Status::Disabled),
        ]
        .spacing(spacing::SM),
    )
    .width(Length::FillPortion(3))
    .into()
}

fn pick_list_state(label: &'static str, status: pick_list::Status) -> Element<'static, Message> {
    row![
        text(label)
            .size(text::Size::Label)
            .class(text::Variant::Muted)
            .width(82),
        pick_list(
            Some(Environment::Staging),
            Environment::ALL,
            |environment| environment.label().to_owned(),
        )
        .on_select(Message::SelectEnvironment)
        .style(move |theme, _| {
            pick_list::appearance(theme, status, pick_list::Variant::Standard)
        })
        .width(Fill),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn combo_box_specimens(gallery: &Gallery) -> Element<'_, Message> {
    panel(
        column![
            text("Searchable environment selector").class(text::Variant::Primary),
            row![
                combo_box(
                    &gallery.environments,
                    "Search environments",
                    Some(&gallery.environment),
                    Message::SelectEnvironment,
                )
                .class(combo_box::Variant::Standard),
                combo_box(
                    &gallery.environments,
                    "Subtle search",
                    None,
                    Message::SelectEnvironment,
                )
                .class(combo_box::Variant::Subtle),
                combo_box(
                    &gallery.environments,
                    "Selection required",
                    None,
                    Message::SelectEnvironment,
                )
                .class(combo_box::Variant::Invalid),
            ]
            .spacing(spacing::SM),
        ]
        .spacing(9),
    )
    .into()
}

fn scrollable_specimens() -> Element<'static, Message> {
    band([
        scroll_panel(
            "Vertical",
            scrollable(
                column![
                    log_line("10:42:01", "Queued build"),
                    log_line("10:42:03", "Resolved dependencies"),
                    log_line("10:42:08", "Compiled workspace"),
                    log_line("10:42:10", "Ran policy checks"),
                    log_line("10:42:12", "Uploaded artifact"),
                    log_line("10:42:14", "Started rollout"),
                ]
                .spacing(7),
            )
            .class(scrollable::Variant::Standard)
            .height(104)
            .width(Fill),
        ),
        scroll_panel(
            "Horizontal",
            scrollable(metrics([
                ("REQUESTS", "1,284"),
                ("ERRORS", "12"),
                ("LATENCY", "86 ms"),
                ("REGIONS", "4"),
            ]))
            .direction(scrollable::Direction::Horizontal(scrollable::scrollbar()))
            .class(scrollable::Variant::Subtle)
            .height(104)
            .width(Fill),
        ),
        scroll_panel(
            "Both axes",
            scrollable(
                container(
                    column![
                        text("Deployment topology")
                            .size(text::Size::Label)
                            .class(text::Variant::Primary),
                        metrics([
                            ("US EAST", "Healthy"),
                            ("US WEST", "Healthy"),
                            ("EU WEST", "Rolling"),
                            ("AP SOUTH", "Queued"),
                        ]),
                    ]
                    .spacing(14),
                )
                .class(container::Variant::Canvas)
                .padding(14)
                .width(580)
                .height(170),
            )
            .direction(scrollable::Direction::Both {
                vertical: scrollable::scrollbar(),
                horizontal: scrollable::scrollbar(),
            })
            .class(scrollable::Variant::Standard)
            .height(104)
            .width(Fill),
        ),
    ])
}

fn scroll_panel(
    title: &'static str,
    content: scrollable::Scrollable<'static, Message>,
) -> Element<'static, Message> {
    panel(
        column![
            text(title)
                .size(text::Size::Label)
                .class(text::Variant::Muted),
            content,
        ]
        .spacing(spacing::SM),
    )
    .into()
}

fn log_line(time: &'static str, event: &'static str) -> Element<'static, Message> {
    row![
        text(time)
            .size(text::Size::Label)
            .class(text::Variant::Muted)
            .width(58),
        text(event)
            .size(text::Size::Label)
            .class(text::Variant::Secondary),
    ]
    .spacing(6)
    .into()
}

fn metrics(items: [(&'static str, &'static str); 4]) -> Element<'static, Message> {
    divided(items.map(|(label, value)| {
        container(
            column![
                text(label)
                    .size(text::Size::Label.sm())
                    .class(text::Variant::Muted),
                text(value)
                    .size(text::Size::Label)
                    .class(text::Variant::Primary),
            ]
            .spacing(spacing::XS),
        )
        .padding([2.0, spacing::MD])
        .width(132)
        .into()
    }))
    .into()
}
