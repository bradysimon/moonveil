//! Authored theme definitions and resolved runtime themes.

use std::{fmt::Display, sync::Arc};

use crate::{Color, Contrast, ResolveError, token::Colors};
use iced_anim::Animate;

mod appearance;

pub use appearance::{Appearance, BorderWidths, Radii, Shadow, Shadows};

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
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
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
    pub profile: Contrast,
    /// The seed colors that derive resolved tokens.
    pub seed: Seed,
}

impl Definition {
    /// Creates a theme definition.
    pub fn new(metadata: Metadata, polarity: Polarity, profile: Contrast, seed: Seed) -> Self {
        Self {
            metadata,
            polarity,
            profile,
            seed,
        }
    }

    /// Returns the default theme [`Definition`] for the given [`Polarity`].
    ///
    /// This returns either "Moonveil Dark" or "Moonveil Light".
    pub fn default_for(polarity: Polarity) -> Self {
        match polarity {
            Polarity::Dark => Definition::new(
                Metadata {
                    name: "Moonveil Dark".into(),
                    description: None,
                },
                Polarity::Dark,
                Contrast::Standard,
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
            ),
            Polarity::Light => Definition::new(
                Metadata {
                    name: "Moonveil Light".into(),
                    description: None,
                },
                Polarity::Light,
                Contrast::Standard,
                Seed {
                    background: Color::from_rgb(0.945, 0.933, 0.902),
                    foreground: Color::from_rgb(0.125, 0.118, 0.106),
                    tint: Color::from_rgb(1.0, 0.996, 0.980),
                    shade: Color::from_rgb(0.180, 0.170, 0.150),
                    accent: Color::from_rgb(0.180, 0.450, 0.570),
                    success: Color::from_rgb(0.250, 0.500, 0.200),
                    warning: Color::from_rgb(0.580, 0.380, 0.080),
                    danger: Color::from_rgb(0.650, 0.200, 0.250),
                    info: Color::from_rgb(0.100, 0.480, 0.450),
                },
            ),
        }
    }

    /// Returns the built-in Moonveil High Contrast theme definition.
    pub fn high_contrast() -> Self {
        Definition::new(
            Metadata {
                name: "Moonveil High Contrast".into(),
                description: Some(
                    "A deliberately sharp palette for strong text, boundaries, and interaction feedback."
                        .into(),
                ),
            },
            Polarity::Dark,
            Contrast::High,
            Seed {
                background: Color::from_rgb8(0x09, 0x0a, 0x0c),
                foreground: Color::from_rgb8(0xff, 0xff, 0xff),
                tint: Color::from_rgb8(0xff, 0xff, 0xff),
                shade: Color::from_rgb8(0x00, 0x00, 0x00),
                accent: Color::from_rgb8(0x8d, 0xd2, 0xff),
                success: Color::from_rgb8(0xa6, 0xe8, 0x9a),
                warning: Color::from_rgb8(0xff, 0xd2, 0x7a),
                danger: Color::from_rgb8(0xff, 0x90, 0x9b),
                info: Color::from_rgb8(0x8c, 0xdb, 0xef),
            },
        )
    }
}

/// A validated theme with fully resolved color tokens.
///
/// Theme animation interpolates resolved color tokens directly. Intermediate
/// frames are temporary visual blends and are not revalidated against either
/// endpoint's contrast profile.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    data: Arc<Data>,
}

impl Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data.definition.metadata.name)
    }
}

/// The internal representation of a [`Theme`].
#[derive(Debug, PartialEq)]
struct Data {
    /// The authored definition used to create this theme.
    definition: Definition,
    /// The resolved color tokens derived from the definition.
    colors: Colors,
    /// The resolved non-layout appearance values.
    appearance: Appearance,
}

impl Theme {
    /// Validates and resolves an authored theme definition.
    pub fn new(definition: Definition) -> Result<Self, ResolveError> {
        let colors = Colors::resolve(&definition)?;
        let appearance = Appearance::resolve(&definition);

        Ok(Self {
            data: Arc::new(Data {
                definition,
                colors,
                appearance,
            }),
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

    /// Returns the theme's resolved non-layout appearance values.
    pub fn appearance(&self) -> &Appearance {
        &self.data.appearance
    }
}

impl iced_core::theme::Base for Theme {
    fn default(preference: iced_core::theme::Mode) -> Self {
        let polarity = match preference {
            iced_core::theme::Mode::Dark => Polarity::Dark,
            iced_core::theme::Mode::None | iced_core::theme::Mode::Light => Polarity::Light,
        };

        Self::new(Definition::default_for(polarity))
            .expect("Built-in theme definitions must resolve")
    }

    fn mode(&self) -> iced_core::theme::Mode {
        match self.definition().polarity {
            Polarity::Dark => iced_core::theme::Mode::Dark,
            Polarity::Light => iced_core::theme::Mode::Light,
        }
    }

    fn base(&self) -> iced_core::theme::Style {
        iced_core::theme::Style {
            background_color: self.colors().surfaces.surface.into(),
            text_color: self.colors().content.primary.into(),
        }
    }

    fn seed(&self) -> Option<iced_core::theme::palette::Seed> {
        let seed = self.definition().seed;

        Some(iced_core::theme::palette::Seed {
            background: seed.background.into(),
            text: seed.foreground.into(),
            primary: seed.accent.into(),
            success: seed.success.into(),
            warning: seed.warning.into(),
            danger: seed.danger.into(),
        })
    }

    fn name(&self) -> &str {
        &self.definition().metadata.name
    }
}

impl iced_anim::Animate for Theme {
    fn components() -> usize {
        <Colors as Animate>::components() + <Appearance as Animate>::components()
    }

    fn update(&mut self, components: &mut impl Iterator<Item = f32>) {
        let mut colors = *self.colors();
        let mut appearance = *self.appearance();
        colors.update(components);
        appearance.update(components);

        self.data = Arc::new(Data {
            definition: self.definition().clone(),
            colors,
            appearance,
        });
    }

    fn distance_to(&self, end: &Self) -> Vec<f32> {
        let mut distance = self.colors().distance_to(end.colors());
        distance.extend(self.appearance().distance_to(end.appearance()));
        distance
    }

    fn lerp(&mut self, start: &Self, end: &Self, progress: f32) {
        let progress = progress.clamp(0.0, 1.0);
        let definition = if progress < 0.5 {
            start.definition().clone()
        } else {
            end.definition().clone()
        };
        let mut colors = *start.colors();
        let mut appearance = *start.appearance();

        colors.lerp(start.colors(), end.colors(), progress);
        appearance.lerp(start.appearance(), end.appearance(), progress);

        self.data = Arc::new(Data {
            definition,
            colors,
            appearance,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Polarity, Seed};
    use iced_core::theme::{Base, Mode};

    fn definition() -> Definition {
        Definition::new(
            Metadata {
                name: "Moonveil".into(),
                description: Some("Test theme".into()),
            },
            Polarity::Dark,
            Contrast::Standard,
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

    /// The default light theme should be constructible without error.
    #[test]
    fn default_light_theme_is_valid() {
        let definition = Definition::default_for(Polarity::Light);
        let theme = Theme::new(definition.clone()).unwrap();
        assert_eq!(theme.definition(), &definition);
        assert_eq!(theme.name(), "Moonveil Light");
    }

    /// The default dark theme should be constructible without error.
    #[test]
    fn default_dark_theme_is_valid() {
        let definition = Definition::default_for(Polarity::Dark);
        let theme = Theme::new(definition.clone()).unwrap();
        assert_eq!(theme.definition(), &definition);
        assert_eq!(theme.name(), "Moonveil Dark");
    }

    #[test]
    fn high_contrast_theme_is_valid() {
        let definition = Definition::high_contrast();
        let theme = Theme::new(definition.clone()).unwrap();

        assert_eq!(theme.definition(), &definition);
        assert_eq!(theme.name(), "Moonveil High Contrast");
        assert_eq!(theme.definition().profile, Contrast::High);
        assert_eq!(
            theme.definition().seed.background,
            Color::from_rgb8(0x09, 0x0a, 0x0c)
        );
        assert_eq!(
            theme.definition().seed.accent,
            Color::from_rgb8(0x8d, 0xd2, 0xff)
        );
    }

    #[test]
    fn resolves_definition_and_exposes_tokens() {
        let definition = definition();
        let expected_background = definition.seed.background;
        let theme = Theme::new(definition.clone()).unwrap();

        assert_eq!(theme.definition(), &definition);
        assert_eq!(theme.colors().surfaces.surface, expected_background);
        assert_eq!(theme.appearance().radius.md, 8.0);
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

    #[test]
    fn iced_base_uses_resolved_theme_tokens() {
        let theme = Theme::new(definition()).unwrap();
        let style = theme.base();

        assert_eq!(
            style.background_color,
            theme.colors().surfaces.surface.into()
        );
        assert_eq!(style.text_color, theme.colors().content.primary.into());
        assert_eq!(theme.mode(), Mode::Dark);
        assert_eq!(theme.name(), theme.definition().metadata.name);
    }

    #[test]
    fn iced_base_resolves_light_and_dark_defaults() {
        let light = <Theme as Base>::default(Mode::Light);
        let dark = <Theme as Base>::default(Mode::Dark);

        assert_eq!(light.mode(), Mode::Light);
        assert_eq!(dark.mode(), Mode::Dark);
        assert_eq!(<Theme as Base>::default(Mode::None), light);
    }

    #[test]
    fn iced_seed_preserves_supported_authored_roles() {
        let theme = Theme::new(definition()).unwrap();
        let iced = <Theme as Base>::seed(&theme).unwrap();
        let seed = theme.definition().seed;

        assert_eq!(iced.background, seed.background.into());
        assert_eq!(iced.text, seed.foreground.into());
        assert_eq!(iced.primary, seed.accent.into());
        assert_eq!(iced.success, seed.success.into());
        assert_eq!(iced.warning, seed.warning.into());
        assert_eq!(iced.danger, seed.danger.into());
    }

    #[test]
    fn theme_animation_interpolates_resolved_tokens() {
        let start = <Theme as Base>::default(Mode::Dark);
        let mut end_definition = start.definition().clone();
        end_definition.metadata.name = "Moonveil Blue".into();
        end_definition.seed.accent = Color::from_rgb(0.361, 0.761, 0.733);
        end_definition.seed.shade = Color::from_rgb(0.080, 0.030, 0.050);
        let end = Theme::new(end_definition).unwrap();
        let mut theme = start.clone();

        theme.lerp(&start, &end, 0.5);

        assert_ne!(theme, start);
        assert_ne!(theme, end);
        assert_eq!(theme.distance_to(&end).len(), Theme::components());

        let mut expected_surface = start.colors().surfaces.surface;
        expected_surface.lerp(
            &start.colors().surfaces.surface,
            &end.colors().surfaces.surface,
            0.5,
        );
        assert_eq!(theme.colors().surfaces.surface, expected_surface);
        let mut expected_shadow = start.appearance().shadow.popover.color;
        expected_shadow.lerp(
            &start.appearance().shadow.popover.color,
            &end.appearance().shadow.popover.color,
            0.5,
        );
        assert_eq!(theme.appearance().shadow.popover.color, expected_shadow);

        theme.lerp(&start, &end, 1.0);
        assert_eq!(theme, end);
    }
}
