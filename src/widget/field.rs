use crate::{
    Color, Theme,
    token::{Interaction, Surface},
};
use iced_core::{Background, Border};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Variant {
    Standard,
    Subtle,
    Code,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Status {
    Active,
    Hovered,
    Focused,
    Disabled,
}

pub(super) struct Appearance {
    pub background: Background,
    pub border: Border,
    pub placeholder: iced_core::Color,
    pub value: iced_core::Color,
    pub selection: iced_core::Color,
}

pub(super) fn appearance(theme: &Theme, status: Status, variant: Variant) -> Appearance {
    if status == Status::Disabled {
        return Appearance {
            background: Background::Color(theme.surface(Surface::Inset).into()),
            border: border(
                theme,
                theme.colors().borders.subtle,
                theme.appearance().border.hairline,
            ),
            placeholder: theme.colors().content.disabled.into(),
            value: theme.colors().content.disabled.into(),
            selection: theme.colors().interaction.selected.into(),
        };
    }

    let surface = match variant {
        Variant::Code => Surface::Sunken,
        Variant::Standard | Variant::Subtle | Variant::Invalid => Surface::Field,
    };
    let background = match status {
        Status::Hovered => theme.interaction_on(surface, Interaction::Hover),
        Status::Active | Status::Focused => theme.surface(surface),
        Status::Disabled => unreachable!(),
    };
    let (border_color, border_width) = match (variant, status) {
        (Variant::Invalid, _) => (
            theme.colors().danger.border,
            if status == Status::Focused {
                theme.appearance().border.focus
            } else {
                theme.appearance().border.control
            },
        ),
        (_, Status::Focused) => (
            theme.colors().borders.focus,
            theme.appearance().border.focus,
        ),
        (Variant::Subtle, _) => (
            theme.colors().borders.standard,
            theme.appearance().border.hairline,
        ),
        (Variant::Standard | Variant::Code, _) => (
            theme.colors().borders.strong,
            theme.appearance().border.control,
        ),
    };

    Appearance {
        background: Background::Color(background.into()),
        border: border(theme, border_color, border_width),
        placeholder: theme.colors().content.muted.into(),
        value: theme.colors().content.primary.into(),
        selection: theme.colors().interaction.selected.into(),
    }
}

fn border(theme: &Theme, color: Color, width: f32) -> Border {
    Border {
        color: color.into(),
        width,
        radius: theme.appearance().radius.sm.into(),
    }
}
