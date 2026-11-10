//! Shared UI elements used in the gallery example.
use iced::Alignment::Center;
use iced::Length::Fill;
use iced::{Alignment, Length};
use moonveil::widget::text::TextExt as _;
use moonveil::widget::{Space, column, labeled, pick_list, row, rule, scrollable, text};
use moonveil::{
    Color, Element, Theme, spacing,
    widget::{Column, Row, button, container},
};

use crate::gallery::Gallery;
use crate::message::Message;
use crate::section::Section;
use crate::sections;
use crate::shared::{Rounding, ThemeChoice};

pub(crate) const COLOR_MODE_ICON: &[u8] = br##"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
    <path d="M15 42 27 17l8 18 5-10 9 17H15Z" fill="#8dd2ff"/>
    <circle cx="45" cy="18" r="7" fill="#ffd27a"/>
    <path d="M18 46h30" stroke="#a6e89a" stroke-width="4" stroke-linecap="round"/>
</svg>
"##;

pub(crate) fn sidebar(gallery: &Gallery) -> Element<'_, Message> {
    let mut navigation = Column::new().spacing(spacing::XS);

    for section in Section::ALL {
        pub fn indicator_style(theme: &Theme, is_active: bool) -> rule::Style {
            rule::Style {
                color: if is_active {
                    theme.colors().accent.foreground
                } else {
                    Color::TRANSPARENT
                }
                .into(),
                radius: theme.appearance().radius.full.into(),
                fill_mode: rule::FillMode::Full,
                snap: true,
            }
        }

        let selected = section == gallery.section;
        navigation = navigation.push(
            button(
                row![
                    container(
                        rule::vertical(3).style(move |theme| indicator_style(theme, selected))
                    )
                    .height(Fill.max(24)),
                    text(section.label()).style(move |theme| text::Style {
                        color: if selected {
                            Some(theme.colors().content.primary.into())
                        } else {
                            None
                        }
                    })
                ]
                .align_y(Center)
                .spacing(spacing::SM),
            )
            .class(button::Variant::Ghost)
            .on_press(Message::SelectSection(section))
            .padding([10.0, spacing::MD])
            .width(Fill),
        );
    }

    container(
        column![
            column![
                text("MOONVEIL")
                    .size(text::Size::Label)
                    .class(text::Variant::Muted),
                text("Widget gallery")
                    .size(text::Size::Headline)
                    .class(text::Variant::Primary)
                    .line_height(1.0),
            ],
            Space::new().height(12),
            text("SECTIONS")
                .size(text::Size::Label)
                .class(text::Variant::Muted),
            navigation,
        ]
        .spacing(spacing::MD),
    )
    .class(container::Variant::Canvas)
    .padding(20)
    .width(230)
    .height(Fill)
    .into()
}

pub(crate) fn workspace(gallery: &Gallery) -> Element<'_, Message> {
    container(column![
        toolbar(gallery),
        rule::horizontal(1).class(rule::Variant::Subtle),
        scrollable(sections::view(gallery))
    ])
    .class(container::Variant::Surface)
    .width(Fill)
    .height(Fill)
    .into()
}

pub(crate) fn toolbar(gallery: &Gallery) -> Element<'_, Message> {
    let theme_picker = pick_list(Some(gallery.theme_choice), ThemeChoice::ALL, |choice| {
        choice.label().to_string()
    })
    .class(pick_list::Variant::Standard)
    .on_select(Message::SelectTheme)
    .width(220);
    let rounding_picker = pick_list(Some(gallery.rounding), Rounding::ALL, |rounding| {
        rounding.label().to_string()
    })
    .class(pick_list::Variant::Standard)
    .on_select(Message::SelectRounding)
    .width(140);

    container(
        row![
            column![
                text(gallery.section.label())
                    .size(text::Size::Headline)
                    .class(text::Variant::Primary),
                text(&gallery.theme.definition().metadata.name).class(text::Variant::Muted),
            ]
            .spacing(3),
            Space::new().width(Fill),
            labeled("Rounding", rounding_picker).reserve_message(false),
            labeled("Theme", theme_picker).reserve_message(false),
        ]
        .spacing(spacing::SM)
        .align_y(Alignment::End),
    )
    .class(container::Variant::Raised)
    .padding([14, 20])
    .width(Fill)
    .into()
}

pub(crate) const PANEL_PADDING: iced::Padding = iced::Padding {
    top: 24.0,
    right: 28.0,
    bottom: 24.0,
    left: 28.0,
};

/// Stacks full-width bands, divided by rules.
pub(crate) fn page<'a>(
    bands: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut page = Column::new().width(Fill);
    for (index, band) in bands.into_iter().enumerate() {
        if index > 0 {
            page = page.push(rule::horizontal(1).class(rule::Variant::Subtle));
        }
        page = page.push(band);
    }
    page.into()
}

/// Places items side by side, divided by rules.
pub(crate) fn divided<'a>(
    items: impl IntoIterator<Item = Element<'a, Message>>,
) -> Row<'a, Message> {
    let mut row = Row::new();
    for (index, item) in items.into_iter().enumerate() {
        if index > 0 {
            row = row.push(rule::vertical(1).class(rule::Variant::Subtle));
        }
        row = row.push(item);
    }
    // Rules fill the height, which would otherwise make the row fill it too.
    row.height(Length::Shrink)
}

pub(crate) fn band<'a>(
    panels: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    divided(panels).width(Fill).into()
}

pub(crate) fn panel<'a>(
    content: impl Into<Element<'a, Message>>,
) -> container::Container<'a, Message> {
    container(content).padding(PANEL_PADDING).width(Fill)
}

pub(crate) fn matrix_label(label: &'static str) -> Element<'static, Message> {
    container(
        text(label)
            .size(text::Size::Label)
            .class(text::Variant::Muted),
    )
    .width(Fill)
    .center_x(Length::Fill)
    .into()
}

pub(crate) fn palette_heading(
    title: &'static str,
    detail: &'static str,
) -> Element<'static, Message> {
    row![
        text(title)
            .size(text::Size::Label)
            .class(text::Variant::Decorative),
        rule::horizontal(1).class(rule::Variant::Subtle),
        text(detail)
            .size(text::Size::Label)
            .class(text::Variant::Muted),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

pub(crate) fn section_heading(
    eyebrow: &'static str,
    title: &'static str,
    detail: Option<&'static str>,
) -> Element<'static, Message> {
    column![
        text(eyebrow)
            .size(text::Size::Label)
            .class(text::Variant::Decorative),
        text(title)
            .size(text::Size::Headline)
            .class(text::Variant::Primary),
        detail.map(|detail| text(detail).class(text::Variant::Secondary)),
    ]
    .spacing(5)
    .into()
}
