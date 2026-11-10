use crate::gallery::Gallery;
use crate::message::Message;
use crate::shared::{Environment, PreviewTab};
use crate::ui::{band, page, panel, section_heading, COLOR_MODE_ICON};
use iced::{Alignment, Length};
use iced::Length::Fill;
use moonveil::widget::{column, row};
use moonveil::widget::text::TextExt as _;
use moonveil::widget::{checkbox, container, radio, rule, slider, svg, tab_bar, text, toggler, Column, Space};
use moonveil::{Element, spacing, token::Intent};

pub(super) fn selection(gallery: &Gallery) -> Element<'_, Message> {
    page([
        panel(section_heading(
            "CONTROL CATALOGS",
            "Controls and selection",
            None,
        ))
        .into(),
        band([live_selection(gallery), svg_specimens()]),
        tab_bar_specimens(gallery),
        band([checkbox_matrix(), radio_matrix(), toggler_matrix()]),
        slider_specimens(gallery),
    ])
}

fn tab_bar_specimens(gallery: &Gallery) -> Element<'_, Message> {
    panel(
        row![
            column![
                text("Full underline")
                    .size(text::Size::Label)
                    .class(text::Variant::Muted),
                tab_bar(
                    PreviewTab::ALL,
                    Some(gallery.preview_tab),
                    Message::SelectPreviewTab,
                )
                .class(tab_bar::Variant::FullUnderline),
            ]
            .spacing(spacing::SM)
            .width(Fill),
            column![
                text("Active underline")
                    .size(text::Size::Label)
                    .class(text::Variant::Muted),
                tab_bar(
                    PreviewTab::ALL,
                    Some(gallery.preview_tab),
                    Message::SelectPreviewTab,
                )
                .class(tab_bar::Variant::ActiveUnderline),
            ]
            .spacing(spacing::SM)
            .width(Fill),
            column![
                text("Rounded")
                    .size(text::Size::Label)
                    .class(text::Variant::Muted),
                tab_bar(
                    PreviewTab::ALL,
                    Some(gallery.preview_tab),
                    Message::SelectPreviewTab,
                )
                .class(tab_bar::Variant::Rounded),
            ]
            .spacing(spacing::SM)
            .width(Fill),
        ]
        .spacing(spacing::LG),
    )
    .into()
}

fn live_selection(gallery: &Gallery) -> Element<'_, Message> {
    let environments =
        Environment::ALL
            .iter()
            .fold(Column::new().spacing(spacing::SM), |col, environment| {
                col.push(
                    radio(
                        environment.label(),
                        *environment,
                        Some(gallery.environment),
                        Message::SelectEnvironment,
                    )
                    .class(radio::Variant::Standard)
                    .text_size(text::Size::Label),
                )
            });

    panel(
        row![
            column![
                text("Release options").class(text::Variant::Primary),
                checkbox(gallery.include_prereleases)
                    .label("Include prerelease builds")
                    .on_toggle(Message::IncludePrereleases)
                    .class(checkbox::Variant::Standard)
                    .text_size(text::Size::Label),
                checkbox(gallery.accepted_terms)
                    .label("Acknowledge production impact")
                    .on_toggle(Message::AcceptTerms)
                    .class(checkbox::Variant::Invalid)
                    .text_size(text::Size::Label),
                text("Required before production deploys")
                    .size(text::Size::Label)
                    .class(text::Variant::Semantic(Intent::Danger)),
            ]
            .spacing(spacing::SM)
            .width(Fill),
            column![
                text("Environment").class(text::Variant::Primary),
                environments,
            ]
            .spacing(spacing::SM)
            .width(Fill),
            column![
                text("Automation").class(text::Variant::Primary),
                toggler(gallery.auto_retry)
                    .label("Retry failed checks")
                    .on_toggle(Message::AutoRetry)
                    .class(toggler::Variant::Standard)
                    .text_size(text::Size::Label),
                toggler(gallery.incident_alerts)
                    .label("Alert on failed deploys")
                    .on_toggle(Message::IncidentAlerts)
                    .class(toggler::Variant::Semantic(Intent::Danger))
                    .text_size(text::Size::Label),
            ]
            .spacing(spacing::SM)
            .width(Fill),
        ]
        .spacing(spacing::LG),
    )
    .width(Length::FillPortion(3))
    .into()
}

fn svg_specimens() -> Element<'static, Message> {
    panel(
        column![
            text("SVG color modes").class(text::Variant::Primary),
            row![
                live_icon("Primary", svg::Variant::Primary),
                live_icon("Muted", svg::Variant::Muted),
                live_icon("Danger", svg::Variant::Semantic(Intent::Danger)),
                live_icon("Original", svg::Variant::Original),
            ]
            .spacing(14)
            .align_y(Alignment::Center),
            rule::horizontal(1).class(rule::Variant::Subtle),
            row![
                text("FORCED")
                    .size(text::Size::Label)
                    .class(text::Variant::Muted)
                    .width(52),
                forced_icon("Idle", svg::Variant::Secondary, svg::Status::Idle),
                forced_icon("Hover", svg::Variant::Secondary, svg::Status::Hovered,),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(spacing::MD),
    )
    .width(Length::FillPortion(2))
    .into()
}

fn live_icon(label: &'static str, variant: svg::Variant) -> Element<'static, Message> {
    column![
        svg(svg::Handle::from_memory(COLOR_MODE_ICON))
            .class(variant)
            .width(32)
            .height(32),
        text(label)
            .size(text::Size::Label)
            .class(text::Variant::Muted),
    ]
    .spacing(5)
    .align_x(Alignment::Center)
    .into()
}

fn forced_icon(
    label: &'static str,
    variant: svg::Variant,
    status: svg::Status,
) -> Element<'static, Message> {
    row![
        svg(svg::Handle::from_memory(COLOR_MODE_ICON))
            .style(move |theme, _| svg::appearance(theme, status, variant))
            .width(20)
            .height(20),
        text(label)
            .size(text::Size::Label)
            .class(text::Variant::Secondary),
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .into()
}

fn checkbox_matrix() -> Element<'static, Message> {
    state_matrix(
        "Checkbox",
        column![
            state_header("STATE", "OFF", "ON"),
            checkbox_state_row("Active", checkbox::Status::Active { is_checked: false }),
            checkbox_state_row("Hovered", checkbox::Status::Hovered { is_checked: false }),
            checkbox_state_row("Disabled", checkbox::Status::Disabled { is_checked: false }),
        ]
        .spacing(spacing::SM),
    )
}

fn checkbox_state_row(label: &'static str, status: checkbox::Status) -> Element<'static, Message> {
    let checked_status = match status {
        checkbox::Status::Active { .. } => checkbox::Status::Active { is_checked: true },
        checkbox::Status::Hovered { .. } => checkbox::Status::Hovered { is_checked: true },
        checkbox::Status::Disabled { .. } => checkbox::Status::Disabled { is_checked: true },
    };

    row![
        text(label)
            .size(text::Size::Label)
            .class(text::Variant::Muted)
            .width(58),
        state_cell(forced_checkbox(false, status)),
        state_cell(forced_checkbox(true, checked_status)),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn forced_checkbox(is_checked: bool, status: checkbox::Status) -> Element<'static, Message> {
    checkbox(is_checked)
        .style(move |theme, _| checkbox::appearance(theme, status, checkbox::Variant::Standard))
        .size(16)
        .into()
}

fn radio_matrix() -> Element<'static, Message> {
    state_matrix(
        "Radio",
        column![
            state_header("STATE", "OFF", "ON"),
            radio_state_row("Active", radio::Status::Active { is_selected: false }),
            radio_state_row("Hovered", radio::Status::Hovered { is_selected: false }),
            text("No disabled or focus status")
                .size(text::Size::Label)
                .class(text::Variant::Muted),
        ]
        .spacing(spacing::SM),
    )
}

fn radio_state_row(label: &'static str, status: radio::Status) -> Element<'static, Message> {
    let selected_status = match status {
        radio::Status::Active { .. } => radio::Status::Active { is_selected: true },
        radio::Status::Hovered { .. } => radio::Status::Hovered { is_selected: true },
    };

    row![
        text(label)
            .size(text::Size::Label)
            .class(text::Variant::Muted)
            .width(58),
        state_cell(forced_radio(false, status)),
        state_cell(forced_radio(true, selected_status)),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn forced_radio(is_selected: bool, status: radio::Status) -> Element<'static, Message> {
    radio("", true, is_selected.then_some(true), |_| Message::Action)
        .style(move |theme, _| radio::appearance(theme, status, radio::Variant::Standard))
        .size(16)
        .into()
}

fn toggler_matrix() -> Element<'static, Message> {
    state_matrix(
        "Toggler",
        column![
            state_header("STATE", "OFF", "ON"),
            toggler_state_row("Active", toggler::Status::Active { is_toggled: false }),
            toggler_state_row("Hovered", toggler::Status::Hovered { is_toggled: false }),
            toggler_state_row("Disabled", toggler::Status::Disabled { is_toggled: false }),
            rule::horizontal(1).class(rule::Variant::Subtle),
            text("SIZES")
                .size(text::Size::Label.sm())
                .class(text::Variant::Muted),
            toggler_size_row("Compact", toggler::size::COMPACT),
            toggler_size_row("Default", toggler::size::DEFAULT),
        ]
        .spacing(spacing::SM),
    )
}

fn toggler_size_row(label: &'static str, size: f32) -> Element<'static, Message> {
    row![
        text(label)
            .size(text::Size::Label)
            .class(text::Variant::Muted)
            .width(58),
        toggler(true).on_toggle(|_| Message::Action).size(size),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn toggler_state_row(label: &'static str, status: toggler::Status) -> Element<'static, Message> {
    let toggled_status = match status {
        toggler::Status::Active { .. } => toggler::Status::Active { is_toggled: true },
        toggler::Status::Hovered { .. } => toggler::Status::Hovered { is_toggled: true },
        toggler::Status::Disabled { .. } => toggler::Status::Disabled { is_toggled: true },
    };

    row![
        text(label)
            .size(text::Size::Label)
            .class(text::Variant::Muted)
            .width(58),
        state_cell(forced_toggler(false, status)),
        state_cell(forced_toggler(true, toggled_status)),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn forced_toggler(is_toggled: bool, status: toggler::Status) -> Element<'static, Message> {
    toggler(is_toggled)
        .style(move |theme, _| toggler::appearance(theme, status, toggler::Variant::Standard))
        .into()
}

fn state_header(
    state: &'static str,
    off: &'static str,
    on: &'static str,
) -> Element<'static, Message> {
    row![
        text(state)
            .size(text::Size::Label.sm())
            .class(text::Variant::Muted)
            .width(58),
        state_cell(
            text(off)
                .size(text::Size::Label.sm())
                .class(text::Variant::Muted)
        ),
        state_cell(
            text(on)
                .size(text::Size::Label.sm())
                .class(text::Variant::Muted)
        ),
    ]
    .spacing(spacing::SM)
    .into()
}

fn state_cell(content: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    container(content).width(Fill).center_x(Length::Fill).into()
}

fn state_matrix(
    title: &'static str,
    content: impl Into<Element<'static, Message>>,
) -> Element<'static, Message> {
    panel(
        column![
            text(title)
                .size(text::Size::Title.sm())
                .class(text::Variant::Primary),
            content.into()
        ]
        .spacing(10),
    )
    .into()
}

fn slider_specimens(gallery: &Gallery) -> Element<'_, Message> {
    band([
        panel(
            column![
                text("Live range controls")
                    .size(text::Size::Title.sm())
                    .class(text::Variant::Primary),
                slider_control(
                    "Canvas zoom",
                    50.0..=200.0,
                    gallery.zoom,
                    "%",
                    slider::Variant::Standard,
                    Message::ZoomChanged,
                ),
                slider_control(
                    "Alert threshold",
                    0.0..=100.0,
                    gallery.alert_threshold,
                    "%",
                    slider::Variant::Semantic(Intent::Warning),
                    Message::AlertThresholdChanged,
                ),
            ]
            .spacing(spacing::MD),
        )
        .width(Length::FillPortion(2))
        .into(),
        panel(
            column![
                text("Explicit states")
                    .size(text::Size::Title.sm())
                    .class(text::Variant::Primary),
                slider_header(),
                slider_state_row("Active", slider::Status::Active),
                slider_state_row("Hovered", slider::Status::Hovered),
                slider_state_row("Dragged", slider::Status::Dragged),
            ]
            .spacing(spacing::SM),
        )
        .width(Length::FillPortion(3))
        .into(),
    ])
}

fn slider_control<'a>(
    label: &'static str,
    range: std::ops::RangeInclusive<f32>,
    value: f32,
    suffix: &'static str,
    variant: slider::Variant,
    on_change: impl Fn(f32) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        row![
            text(label)
                .size(text::Size::Label)
                .class(text::Variant::Secondary),
            Space::new().width(Fill),
            text(format!("{value:.0}{suffix}"))
                .size(text::Size::Label)
                .class(text::Variant::Muted),
        ]
        .align_y(Alignment::Center),
        slider(range, value, on_change).class(variant).step(1.0),
    ]
    .spacing(5)
    .into()
}

fn slider_header() -> Element<'static, Message> {
    row![
        text("STATE")
            .size(text::Size::Label.sm())
            .class(text::Variant::Muted)
            .width(58),
        state_cell(
            text("MIN")
                .size(text::Size::Label.sm())
                .class(text::Variant::Muted)
        ),
        state_cell(
            text("MID")
                .size(text::Size::Label.sm())
                .class(text::Variant::Muted)
        ),
        state_cell(
            text("MAX")
                .size(text::Size::Label.sm())
                .class(text::Variant::Muted)
        ),
    ]
    .spacing(spacing::SM)
    .into()
}

fn slider_state_row(label: &'static str, status: slider::Status) -> Element<'static, Message> {
    row![
        text(label)
            .size(text::Size::Label)
            .class(text::Variant::Muted)
            .width(58),
        forced_slider(0.0, status),
        forced_slider(50.0, status),
        forced_slider(100.0, status),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn forced_slider(value: f32, status: slider::Status) -> Element<'static, Message> {
    slider(0.0..=100.0, value, |_| Message::Action)
        .style(move |theme, _| slider::appearance(theme, status, slider::Variant::Standard))
        .width(Fill)
        .into()
}
