use crate::gallery::Gallery;
use crate::message::Message;
use crate::ui::{band, page, panel, section_heading, COLOR_MODE_ICON, PANEL_PADDING};
use iced::Alignment;
use iced::Length::Fill;
use iced::widget::{column, row};
use moonveil::widget::text::TextExt as _;
use moonveil::widget::{
    button, container, context_menu, dropdown, popover, rule, snackbar, svg, text, toggler, tooltip,
    Space,
};
use moonveil::{Element, spacing, token::Intent};

pub(super) fn overlays(gallery: &Gallery) -> Element<'_, Message> {
    page([
        panel(section_heading(
            "OVERLAY CATALOGS",
            "Deployment controls",
            Some("A focused release workspace for transient feedback, action menus, and modal confirmation."),
        ))
        .into(),
        deployment_summary(gallery),
        band([feedback_panel(), action_menu_panel()]),
        tooltip_panel(),
        popover_panel(gallery),
    ])
}

fn deployment_summary(gallery: &Gallery) -> Element<'_, Message> {
    container(
        row![
            container(text("PROD").size(text::Size::Label))
                .class(container::Variant::Semantic {
                    intent: Intent::Success,
                    style: container::SemanticStyle::Soft,
                })
                .padding([6, 9]),
            column![
                text("Production deployment")
                    .size(text::Size::Title)
                    .class(text::Variant::Primary),
                text("main  •  us-east-1  •  rolling")
                    .size(text::Size::Label)
                    .class(text::Variant::Muted),
            ]
            .spacing(4),
            Space::new().width(Fill),
            column![
                text(if gallery.dialog.is_visible() {
                    "AWAITING CONFIRMATION"
                } else {
                    "READY TO DEPLOY"
                })
                .size(text::Size::Label.sm())
                .class(text::Variant::Decorative),
                text("Theme preview ready")
                    .size(text::Size::Label)
                    .class(text::Variant::Secondary),
            ]
            .spacing(4)
            .align_x(Alignment::End),
            button(text("Review and deploy").size(text::Size::Label))
                .class(button::Variant::Primary)
                .padding([9, 14])
                .on_press(Message::OpenDialog),
        ]
        .spacing(spacing::MD)
        .align_y(Alignment::Center),
    )
    .class(container::Variant::Raised)
    .padding(PANEL_PADDING)
    .width(Fill)
    .into()
}

fn feedback_panel() -> Element<'static, Message> {
    panel(
        column![
            text("01  TRANSIENT FEEDBACK")
                .size(text::Size::Label)
                .class(text::Variant::Decorative),
            text("Notify the team")
                .size(text::Size::Title)
                .class(text::Variant::Primary),
            text("Preview each intent as a live snackbar notification.")
                .size(text::Size::Label)
                .class(text::Variant::Muted),
            rule::horizontal(1).class(rule::Variant::Subtle),
            column![
                row![
                    snackbar_button("Primary", snackbar::Variant::Primary),
                    snackbar_button("Secondary", snackbar::Variant::Secondary),
                    snackbar_button("Success", snackbar::Variant::Semantic(Intent::Success),),
                ]
                .spacing(spacing::SM),
                row![
                    snackbar_button("Warning", snackbar::Variant::Semantic(Intent::Warning),),
                    snackbar_button("Danger", snackbar::Variant::Semantic(Intent::Danger)),
                    snackbar_button("Accent", snackbar::Variant::Semantic(Intent::Accent)),
                ]
                .spacing(spacing::SM),
            ]
            .spacing(spacing::SM),
        ]
        .spacing(spacing::SM),
    )
    .into()
}

fn snackbar_button(label: &str, variant: snackbar::Variant) -> Element<'_, Message> {
    let button_variant = match variant {
        snackbar::Variant::Primary => button::Variant::Primary,
        snackbar::Variant::Secondary => button::Variant::Secondary,
        snackbar::Variant::Semantic(intent) => button::Variant::Semantic {
            intent,
            style: button::SemanticStyle::Soft,
        },
        _ => button::Variant::Secondary,
    };

    button(text(label).size(text::Size::Label))
        .class(button_variant)
        .padding([7, 10])
        .width(Fill)
        .on_press(Message::ShowSnackbar(variant))
        .into()
}

fn action_menu_panel() -> Element<'static, Message> {
    panel(
        column![
            text("02  CONTEXTUAL ACTIONS")
                .size(text::Size::Label)
                .class(text::Variant::Decorative),
            text("Release commands")
                .size(text::Size::Title)
                .class(text::Variant::Primary),
            text("Open either menu to inspect hierarchy and placement.")
                .size(text::Size::Label)
                .class(text::Variant::Muted),
            rule::horizontal(1).class(rule::Variant::Subtle),
            row![
                dropdown(
                    text("Release menu  ↓"),
                    [
                        dropdown::section("Release"),
                        dropdown::item("Deploy preview", Message::Action,),
                        dropdown::item("Promote to staging", Message::Action,),
                        dropdown::separator(),
                        dropdown::item("Rollback release", Message::Action),
                    ],
                )
                .width(158),
                dropdown(
                    text("Configuration  ↓"),
                    [
                        dropdown::section("Environment"),
                        dropdown::item("Edit variables", Message::Action,),
                        dropdown::item("View audit trail", Message::Action,),
                    ],
                )
                .class(dropdown::Variant::Subtle)
                .menu_class(dropdown::menu::Variant::Subtle)
                .width(158),
            ]
            .spacing(spacing::SM),
            context_menu(
                container(
                    text("Right-click for artifact actions")
                        .size(text::Size::Label)
                        .class(text::Variant::Muted),
                )
                .class(container::Variant::Sunken.floating())
                .padding([10, 12])
                .width(Fill),
                [
                    context_menu::item("Open build log", Message::Action).shortcut(
                        context_menu::Shortcut::command(context_menu::Key::Character('l')),
                    ),
                    context_menu::item("Copy artifact URL", Message::Action).shortcut(
                        context_menu::Shortcut::shift_command(context_menu::Key::Character('c')),
                    ),
                    context_menu::separator(),
                    context_menu::submenu(
                        "Promote to",
                        [
                            context_menu::item("Staging", Message::Action),
                            context_menu::item("Production", Message::Action),
                            context_menu::item("Canary", Message::Action).disabled(true),
                        ],
                    ),
                    context_menu::submenu(
                        "Deploy to region",
                        DEPLOY_REGIONS
                            .iter()
                            .map(|region| context_menu::item(*region, Message::Action)),
                    ),
                    context_menu::separator(),
                    context_menu::item_maybe("Delete artifact", None),
                ],
            ),
        ]
        .spacing(spacing::SM),
    )
    .into()
}

// Long enough to exceed the context menu's max height so the submenu scrolls.
const DEPLOY_REGIONS: &[&str] = &[
    "us-east-1",
    "us-east-2",
    "us-west-1",
    "us-west-2",
    "ca-central-1",
    "sa-east-1",
    "eu-west-1",
    "eu-west-2",
    "eu-west-3",
    "eu-central-1",
    "eu-north-1",
    "eu-south-1",
    "me-south-1",
    "af-south-1",
    "ap-south-1",
    "ap-east-1",
    "ap-northeast-1",
    "ap-northeast-2",
    "ap-southeast-1",
    "ap-southeast-2",
];

fn tooltip_panel() -> Element<'static, Message> {
    panel(
        column![
            text("03  TOOLTIP PLACEMENT")
                .size(text::Size::Label)
                .class(text::Variant::Decorative),
            text("Context at the point of action")
                .size(text::Size::Title)
                .class(text::Variant::Primary),
            text("Hover each control to compare placement and viewport behavior.")
                .size(text::Size::Label)
                .class(text::Variant::Muted),
            rule::horizontal(1).class(rule::Variant::Subtle),
            row![
                tooltip_button("Above", "Appears above the anchor", tooltip::Position::Top),
                tooltip_button(
                    "Below",
                    "Appears below the anchor",
                    tooltip::Position::Bottom,
                ),
                tooltip_button(
                    "Left",
                    "Appears left of the anchor",
                    tooltip::Position::Left
                ),
                tooltip_button(
                    "Right",
                    "Appears right of the anchor",
                    tooltip::Position::Right,
                ),
                tooltip_button(
                    "Follow",
                    "Tracks the pointer",
                    tooltip::Position::FollowCursor,
                ),
                tooltip(
                    button(
                        svg(svg::Handle::from_memory(COLOR_MODE_ICON))
                            .class(svg::Variant::Original)
                            .width(18)
                            .height(18),
                    )
                    .class(button::Variant::Icon)
                    .padding(6)
                    .on_press(Message::Action),
                    text("Immediate icon tooltip").size(text::Size::Label),
                    tooltip::Position::Bottom,
                )
                .delay(iced::time::Duration::ZERO),
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center),
        ]
        .spacing(spacing::SM),
    )
    .into()
}

fn tooltip_button(
    label: &'static str,
    hint: &'static str,
    position: tooltip::Position,
) -> Element<'static, Message> {
    tooltip(
        button(text(label))
            .class(button::Variant::Outline)
            .padding([7, 10])
            .on_press(Message::Action),
        text(hint).size(text::Size::Label),
        position,
    )
    .into()
}

fn popover_panel(gallery: &Gallery) -> Element<'_, Message> {
    let settings = column![
        text("Rollout settings").class(text::Variant::Primary),
        text("Applies to the next production deploy.")
            .size(text::Size::Label)
            .class(text::Variant::Muted),
        rule::horizontal(1).class(rule::Variant::Subtle),
        toggler(gallery.auto_retry)
            .label("Retry failed checks")
            .on_toggle(Message::AutoRetry)
            .text_size(text::Size::Label),
        toggler(gallery.incident_alerts)
            .label("Alert on failed deploys")
            .on_toggle(Message::IncidentAlerts)
            .text_size(text::Size::Label),
    ]
    .spacing(spacing::SM);

    panel(
        column![
            text("04  POPOVER PLACEMENT")
                .size(text::Size::Label)
                .class(text::Variant::Decorative),
            text("Anchored detail panels")
                .size(text::Size::Title)
                .class(text::Variant::Primary),
            text("Press a control to open its popover. Press outside or Esc to dismiss.")
                .size(text::Size::Label)
                .class(text::Variant::Muted),
            rule::horizontal(1).class(rule::Variant::Subtle),
            row![
                popover(
                    button(text("Rollout  ↓"))
                        .class(button::Variant::Secondary)
                        .padding([7, 10])
                        .on_press(Message::Action),
                    settings,
                )
                .width(260),
                popover_button("Top", popover::Position::Top),
                popover_button("Bottom end", popover::Position::BottomEnd),
                popover_button("Left", popover::Position::Left),
                popover_button("Right start", popover::Position::RightStart),
                popover(
                    button(text("Subtle"))
                        .class(button::Variant::Ghost)
                        .padding([7, 10])
                        .on_press(Message::Action),
                    text("A quieter edge for dense surfaces.").size(text::Size::Label),
                )
                .position(popover::Position::Bottom)
                .padding([spacing::SM, spacing::MD])
                .class(popover::Variant::Subtle),
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center),
        ]
        .spacing(spacing::SM),
    )
    .into()
}

fn popover_button(label: &'static str, position: popover::Position) -> Element<'static, Message> {
    popover(
        button(text(label))
            .class(button::Variant::Outline)
            .padding([7, 10])
            .on_press(Message::Action),
        column![
            text(label).class(text::Variant::Primary),
            text("Popovers flip and clamp to stay inside the window.")
                .size(text::Size::Label)
                .class(text::Variant::Muted),
        ]
        .spacing(spacing::XS),
    )
    .position(position)
    .max_width(220)
    .into()
}
