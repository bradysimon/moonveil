//! Authored theme definitions.

use crate::{Color, Profile};

/// Human-readable information about a theme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    /// The display name of the theme.
    pub name: String,
    /// An optional description of the theme's visual character.
    pub description: Option<String>,
}

/// Whether a theme is designed around dark or light surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    Dark,
    Light,
}

/// Seed colors from which resolved tokens are derived.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seed {
    pub background: Color,
    pub foreground: Color,
    pub tint: Color,
    pub shade: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub info: Color,
}

/// Authored source data for a Moonveil theme.
#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    /// Human-readable information about the theme.
    pub metadata: Metadata,
    /// The polarity of the theme, either dark or light.
    pub polarity: Polarity,
    /// The contrast profile used to derive resolved tokens.
    pub profile: Profile,
    /// The seed colors that derive resolved tokens.
    pub seed: Seed,
}

impl Definition {
    /// Creates a theme definition.
    pub fn new(metadata: Metadata, polarity: Polarity, profile: Profile, seed: Seed) -> Self {
        Self {
            metadata,
            polarity,
            profile,
            seed,
        }
    }
}
