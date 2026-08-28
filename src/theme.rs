//! Authored theme definitions and resolved runtime themes.

use std::{fmt::Display, sync::Arc};

use crate::{Color, Profile, ResolveError, token::Colors};

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

/// A validated theme with fully resolved color tokens.
#[derive(Debug, Clone)]
pub struct Theme {
    data: Arc<Data>,
}

impl Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data.definition.metadata.name)
    }
}

/// The internal representation of a [`Theme`].
#[derive(Debug)]
struct Data {
    /// The authored definition used to create this theme.
    definition: Definition,
    /// The resolved color tokens derived from the definition.
    colors: Colors,
}

impl Theme {
    /// Validates and resolves an authored theme definition.
    pub fn new(definition: Definition) -> Result<Self, ResolveError> {
        let colors = Colors::resolve(&definition)?;

        Ok(Self {
            data: Arc::new(Data { definition, colors }),
        })
    }

    /// Returns the authored definition used to create this theme.
    pub fn definition(&self) -> &Definition {
        &self.data.definition
    }

    /// Returns the theme's resolved color tokens.
    pub fn colors(&self) -> &Colors {
        &self.data.colors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Polarity, Seed};

    fn definition() -> Definition {
        Definition::new(
            Metadata {
                name: "Moonveil".into(),
                description: Some("Test theme".into()),
            },
            Polarity::Dark,
            Profile::Standard,
            Seed {
                background: Color::from_rgb(0.098, 0.106, 0.125),
                foreground: Color::from_rgb(0.902, 0.882, 0.835),
                tint: Color::from_rgb(0.961, 0.945, 0.910),
                shade: Color::from_rgb(0.031, 0.035, 0.047),
                accent: Color::from_rgb(0.541, 0.714, 0.784),
                success: Color::from_rgb(0.561, 0.741, 0.525),
                warning: Color::from_rgb(0.875, 0.706, 0.404),
                danger: Color::from_rgb(0.875, 0.486, 0.525),
                info: Color::from_rgb(0.361, 0.761, 0.733),
            },
        )
    }

    #[test]
    fn resolves_definition_and_exposes_tokens() {
        let definition = definition();
        let expected_background = definition.seed.background;
        let theme = Theme::new(definition.clone()).unwrap();

        assert_eq!(theme.definition(), &definition);
        assert_eq!(theme.colors().surfaces.surface, expected_background);
    }

    #[test]
    fn cloning_reuses_resolved_data() {
        let theme = Theme::new(definition()).unwrap();
        let cloned = theme.clone();
        assert!(Arc::ptr_eq(&theme.data, &cloned.data));
    }

    #[test]
    fn returns_resolution_errors_without_constructing_a_theme() {
        let mut definition = definition();
        definition.seed.accent = Color::from_rgba(0.5, 0.7, 0.8, 0.5);
        assert!(matches!(
            Theme::new(definition),
            Err(ResolveError::NonOpaqueSeed { seed: "accent", .. })
        ));
    }

    #[test]
    fn theme_display_uses_name() {
        let definition = definition();
        let theme = Theme::new(definition.clone()).unwrap();
        assert_eq!(theme.to_string(), definition.metadata.name);
    }
}
