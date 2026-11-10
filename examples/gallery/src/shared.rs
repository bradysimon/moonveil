//! Shared types used throughout the gallery example.
use iced::border::Radius;
use moonveil::{Radii, Theme};

use crate::preview_themes;

#[derive(Clone, Copy)]
pub enum Edge {
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Edge {
    fn top_left(&self, amount: f32) -> f32 {
        match self {
            Edge::Left => amount,
            Edge::TopLeft => amount,
            _ => 0.0,
        }
    }

    fn top_right(&self, amount: f32) -> f32 {
        match self {
            Edge::Right => amount,
            Edge::TopRight => amount,
            _ => 0.0,
        }
    }

    fn bottom_left(&self, amount: f32) -> f32 {
        match self {
            Edge::Left => amount,
            Edge::BottomLeft => amount,
            _ => 0.0,
        }
    }

    fn bottom_right(&self, amount: f32) -> f32 {
        match self {
            Edge::Right => amount,
            Edge::BottomRight => amount,
            _ => 0.0,
        }
    }

    pub(crate) fn radius(&self, amount: f32) -> Radius {
        Radius {
            top_left: self.top_left(amount),
            top_right: self.top_right(amount),
            bottom_left: self.bottom_left(amount),
            bottom_right: self.bottom_right(amount),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Field {
    RequestName,
    Search,
    Endpoint,
    CodeValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    pub const ALL: [Self; 3] = [Self::Development, Self::Staging, Self::Production];

    pub fn label(self) -> &'static str {
        match self {
            Self::Development => "Development",
            Self::Staging => "Staging",
            Self::Production => "Production",
        }
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewTab {
    Overview,
    Activity,
    Settings,
}

impl PreviewTab {
    pub const ALL: [Self; 3] = [Self::Overview, Self::Activity, Self::Settings];
}

impl std::fmt::Display for PreviewTab {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Overview => "Overview",
            Self::Activity => "Activity",
            Self::Settings => "Settings",
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    Dark,
    Light,
    HighContrast,
    Everforest,
    RosePine,
    CatppuccinLatte,
    Nord,
}

impl ThemeChoice {
    pub const ALL: [Self; 7] = [
        Self::Dark,
        Self::Light,
        Self::HighContrast,
        Self::Everforest,
        Self::RosePine,
        Self::CatppuccinLatte,
        Self::Nord,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::HighContrast => "High contrast",
            Self::Everforest => "Everforest Light",
            Self::RosePine => "Rosé Pine",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::Nord => "Nord",
        }
    }

    pub fn resolve(self, rounding: Rounding) -> Theme {
        let theme = match self {
            Self::Dark => Theme::default_dark(),
            Self::Light => Theme::default_light(),
            Self::HighContrast => Theme::default_high_contrast(),
            Self::Everforest => preview_themes::everforest(),
            Self::RosePine => preview_themes::rose_pine(),
            Self::CatppuccinLatte => preview_themes::catppuccin_latte(),
            Self::Nord => preview_themes::nord(),
        };
        let radius = rounding.radii();

        if theme.definition().radius == radius {
            return theme;
        }

        Theme::new(theme.definition().clone().with_radius(radius))
            .expect("changing radii must not affect color resolution")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
    Sharp,
    Standard,
    Round,
}

impl Rounding {
    pub const ALL: [Self; 3] = [Self::Sharp, Self::Standard, Self::Round];

    pub fn label(self) -> &'static str {
        match self {
            Self::Sharp => "Sharp",
            Self::Standard => "Standard",
            Self::Round => "Round",
        }
    }

    pub fn radii(self) -> Radii {
        match self {
            Self::Sharp => Radii::SHARP,
            Self::Standard => Radii::STANDARD,
            Self::Round => Radii::ROUND,
        }
    }
}
