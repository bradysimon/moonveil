use crate::gallery::Gallery;
use crate::message::Message;
use crate::shared::Edge;
use crate::ui::{PANEL_PADDING, page, palette_heading, panel};
use iced::Length::Fill;
use iced::{Border, Font, font};
use moonveil::Widget;
use moonveil::widget::text::TextExt as _;
use moonveil::widget::{Space, container, text};
use moonveil::widget::{column, row};
use moonveil::{Color, Theme, spacing};

pub(super) fn overview(gallery: &Gallery) -> impl Widget<Message> + '_ {
    let definition = gallery.theme.definition();
    let seed = definition.seed;
    let description = definition
        .metadata
        .description
        .as_deref()
        .unwrap_or("A contrast-aware palette from resolved seed colors.");

    page([
        container(
            column![
                text("ACTIVE PALETTE")
                    .size(text::Size::Label)
                    .class(text::Variant::Decorative),
                text(definition.metadata.name.as_str())
                    .size(text::Size::Headline.lg())
                    .class(text::Variant::Primary)
                    .line_height(1.0),
                text(description)
                    .size(text::Size::Title)
                    .class(text::Variant::Secondary),
                row![
                    palette_bar(seed.accent, Some(Edge::Left)),
                    palette_bar(seed.success, None),
                    palette_bar(seed.warning, None),
                    palette_bar(seed.danger, Some(Edge::Right)),
                ],
            ]
            .spacing(spacing::MD),
        )
        .class(container::Variant::Raised)
        .padding(PANEL_PADDING)
        .width(Fill)
        .boxed(),
        panel(
            column![
                palette_heading("SEED COLORS", "The authored colors that shape this theme"),
                column![
                    row![
                        color_swatch("Background", seed.background, Some(Edge::TopLeft), false),
                        color_swatch("Foreground", seed.foreground, None, false),
                        color_swatch("Tint", seed.tint, None, false),
                        color_swatch("Shade", seed.shade, Some(Edge::TopRight), false),
                    ],
                    row![
                        color_swatch("Accent", seed.accent, Some(Edge::BottomLeft), false),
                        color_swatch("Success", seed.success, None, false),
                        color_swatch("Warning", seed.warning, None, false),
                        color_swatch("Danger", seed.danger, Some(Edge::BottomRight), false),
                    ],
                ],
            ]
            .spacing(spacing::SM),
        )
        .boxed(),
        panel(categorical_swatches(gallery)).boxed(),
    ])
}

fn categorical_swatches(gallery: &Gallery) -> impl Widget<Message> + '_ {
    let categorical = gallery.theme.colors().categorical;
    let authored = categorical.authored().len();
    let edge = |index| match index {
        0 => Some(Edge::TopLeft),
        3 => Some(Edge::TopRight),
        4 => Some(Edge::BottomLeft),
        7 => Some(Edge::BottomRight),
        _ => None,
    };
    let swatch = |index: usize| {
        let pair = categorical.all()[index].solid.active;
        let origin = if index < authored {
            "Authored"
        } else {
            "Generated"
        };
        let title = text(format!("Slot {}", index + 1)).font(Font {
            weight: font::Weight::Semibold,
            ..Default::default()
        });
        container(column![
            Space::new().height(36),
            title,
            text(origin).size(text::Size::Label),
            text(color_hex(pair.color)),
        ])
        .style(move |theme| color_style(pair.color, pair.text, theme, edge(index), false))
        .padding(10)
        .width(Fill)
        .boxed()
    };

    column![
        palette_heading(
            "CATEGORICAL",
            "Distinct colors for tags, series, and labels"
        ),
        column![row((0..4).map(swatch)), row((4..8).map(swatch))],
    ]
    .spacing(spacing::SM)
}

fn palette_bar(color: Color, edge: Option<Edge>) -> impl Widget<Message> + 'static {
    let text_color = color.on_color();
    container(Space::new())
        .style(move |theme| color_style(color, text_color, theme, edge, true))
        .height(10)
        .width(Fill)
}

fn color_swatch<'a>(
    label: &'static str,
    color: Color,
    edge: Option<Edge>,
    sharp: bool,
) -> impl Widget<Message> + 'a {
    let spacing = Space::new().height(54);
    let title = text(label).font(Font {
        weight: font::Weight::Semibold,
        ..Default::default()
    });
    let text_color = color.on_color();
    container(column![spacing, title, text(color_hex(color))])
        .style(move |theme| color_style(color, text_color, theme, edge, sharp))
        .padding(10)
        .width(Fill)
        .height(112)
}

fn color_style(
    color: Color,
    text_color: Color,
    theme: &Theme,
    edge: Option<Edge>,
    sharp: bool,
) -> container::Style {
    let radius = if sharp {
        theme.appearance().radius.xs
    } else {
        theme.appearance().radius.md
    };
    container::Style {
        text_color: Some(text_color.into()),
        border: Border::default().rounded(edge.map(|e| e.radius(radius)).unwrap_or_default()),
        background: Some(iced::Background::Color(color.into())),
        ..container::Style::default()
    }
}

fn color_hex(color: Color) -> String {
    let [red, green, blue, _] = color.components();
    format!(
        "#{:02X}{:02X}{:02X}",
        (red * 255.0).round() as u8,
        (green * 255.0).round() as u8,
        (blue * 255.0).round() as u8,
    )
}
