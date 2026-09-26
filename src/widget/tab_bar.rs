//! Horizontal tab navigation helpers.

use crate::{
    Color, Element, Theme,
    widget::{container, stack, text},
};
use iced_core::{Alignment, Background, Border, Length, Padding};
use iced_widget::{Row, button, column, rule};

pub use iced_widget::button::{Status, Style};

/// Creates a tab bar from selectable options.
pub fn tab_bar<'a, T, Message>(
    options: impl IntoIterator<Item = T>,
    selected: Option<T>,
    on_select: impl Fn(T) -> Message + 'a,
) -> TabBar<'a, T, Message>
where
    T: Clone + PartialEq + ToString + 'a,
    Message: Clone + 'a,
{
    TabBar::new(options, selected, on_select)
}

/// A horizontal collection of mutually exclusive tabs.
pub struct TabBar<'a, T, Message>
where
    T: Clone + PartialEq + ToString + 'a,
    Message: Clone + 'a,
{
    options: Vec<T>,
    selected: Option<T>,
    on_select: Box<dyn Fn(T) -> Message + 'a>,
    variant: Variant,
    spacing: f32,
    padding: Padding,
    width: Length,
}

impl<'a, T, Message> TabBar<'a, T, Message>
where
    T: Clone + PartialEq + ToString + 'a,
    Message: Clone + 'a,
{
    /// Creates a tab bar from selectable options.
    pub fn new(
        options: impl IntoIterator<Item = T>,
        selected: Option<T>,
        on_select: impl Fn(T) -> Message + 'a,
    ) -> Self {
        Self {
            options: options.into_iter().collect(),
            selected,
            on_select: Box::new(on_select),
            variant: Variant::default(),
            spacing: 4.0,
            padding: Padding::from([4.0, 8.0]),
            width: Length::Shrink,
        }
    }

    /// Sets the visual treatment of the tabs.
    #[must_use]
    pub fn class(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the spacing between tabs.
    #[must_use]
    pub fn spacing(mut self, spacing: impl Into<iced_core::Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }

    /// Sets the padding within each tab.
    #[must_use]
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the complete tab bar.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

/// A built-in Moonveil tab bar style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// Draws an accent underline beneath only the selected tab.
    #[default]
    ActiveUnderline,
    /// Draws a full rail with an accent segment beneath the selected tab.
    FullUnderline,
    /// Marks the selected tab with a rounded background.
    Rounded,
}

/// Returns the resolved button style for a tab and interaction status.
pub fn appearance(theme: &Theme, status: Status, variant: Variant, is_selected: bool) -> Style {
    let text_color = text_color(theme, status, is_selected);
    let background = match variant {
        Variant::FullUnderline | Variant::ActiveUnderline => None,
        Variant::Rounded => is_selected.then_some(theme.colors().interaction.selected),
    }
    .map(|color| Background::Color(color.into()));

    Style {
        background,
        text_color: text_color.into(),
        border: Border {
            radius: match variant {
                Variant::FullUnderline | Variant::ActiveUnderline => 0.0,
                Variant::Rounded => theme.appearance().radius.sm,
            }
            .into(),
            ..Border::default()
        },
        ..Style::default()
    }
}

fn text_color(theme: &Theme, status: Status, is_selected: bool) -> Color {
    if status == Status::Disabled {
        return theme.colors().content.disabled;
    }

    if is_selected || status == Status::Hovered {
        theme.colors().content.primary
    } else {
        theme.colors().content.muted
    }
}

fn indicator_style(theme: &Theme, variant: Variant, is_selected: bool) -> rule::Style {
    rule::Style {
        color: if is_selected {
            theme.colors().accent.indicator.into()
        } else {
            iced_core::Color::TRANSPARENT
        },
        radius: match variant {
            Variant::FullUnderline => theme.appearance().radius.xs,
            Variant::ActiveUnderline | Variant::Rounded => theme.appearance().radius.full,
        }
        .into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

fn rail_style(theme: &Theme) -> rule::Style {
    rule::Style {
        color: theme.colors().borders.subtle.into(),
        radius: theme.appearance().radius.xs.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

fn tab_spacing(variant: Variant, spacing: f32) -> f32 {
    match variant {
        Variant::FullUnderline => 0.0,
        Variant::ActiveUnderline | Variant::Rounded => spacing,
    }
}

impl<'a, T, Message> From<TabBar<'a, T, Message>> for Element<'a, Message>
where
    T: Clone + PartialEq + ToString + 'a,
    Message: Clone + 'a,
{
    fn from(tab_bar: TabBar<'a, T, Message>) -> Self {
        let selected = tab_bar.selected;
        let on_select = tab_bar.on_select;
        let variant = tab_bar.variant;
        let padding = tab_bar.padding;
        let width = tab_bar.width;
        let tabs = tab_bar.options.into_iter().map(move |option| {
            let is_selected = selected.as_ref() == Some(&option);
            let label = text(option.to_string()).size(text::size::BODY);
            let button_padding = match variant {
                Variant::FullUnderline => Padding {
                    top: padding.top,
                    right: 0.0,
                    bottom: padding.bottom,
                    left: 0.0,
                },
                Variant::ActiveUnderline | Variant::Rounded => padding,
            };
            let content: Element<'a, Message> = match variant {
                Variant::FullUnderline => column![
                    container(label).padding(Padding {
                        top: 0.0,
                        right: padding.right,
                        bottom: 0.0,
                        left: padding.left,
                    }),
                    rule::horizontal(2).style(move |theme| indicator_style(
                        theme,
                        variant,
                        is_selected
                    )),
                ]
                .spacing(4)
                .into(),
                Variant::ActiveUnderline => column![
                    label,
                    rule::horizontal(2).style(move |theme| indicator_style(
                        theme,
                        variant,
                        is_selected
                    )),
                ]
                .spacing(4)
                .into(),
                Variant::Rounded => label.into(),
            };

            button(content)
                .on_press(on_select(option))
                .padding(button_padding)
                .style(move |theme, status| appearance(theme, status, variant, is_selected))
                .into()
        });

        let tabs = Row::from_iter(tabs)
            .spacing(tab_spacing(variant, tab_bar.spacing))
            .width(width);

        match variant {
            Variant::FullUnderline => stack([tabs.into()])
                .push_under(
                    container(rule::horizontal(2).style(rail_style))
                        .padding(Padding {
                            top: 0.0,
                            right: 0.0,
                            bottom: padding.bottom,
                            left: 0.0,
                        })
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_y(Alignment::End),
                )
                .width(width)
                .into(),
            Variant::ActiveUnderline | Variant::Rounded => tabs.into(),
        }
    }
}
