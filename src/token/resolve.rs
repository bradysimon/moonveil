use crate::{
    Color,
    color::Oklch,
    contrast::{adjust_foreground, composite, contrast_ratio},
    theme::{Definition, Polarity},
};

use super::{
    BorderRole, Borders, Colors, Content, ContentRole, Intent, Interaction, ResolveError, Surfaces,
    TokenRole, semantic::Resolver as SemanticResolver,
};

const MINIMUM_SURFACE_LIGHTNESS_DELTA: f32 = 0.01;
const CONTRAST_EPSILON: f32 = 0.000_1;

impl Colors {
    /// Derives and validates all color tokens for an authored theme definition.
    pub fn resolve(definition: &Definition) -> Result<Self, ResolveError> {
        validate_definition(definition)?;

        let targets = definition.profile.targets();
        let high_emphasis = targets.normal_text >= 7.0 || targets.boundary >= 4.5;
        let surfaces = resolve_surfaces(definition);
        validate_surfaces(&surfaces, definition.polarity)?;

        let neutral = neutral_backgrounds(&surfaces);
        let neutral_colors = neutral.map(|(_, color)| color);
        let faded = |alpha| {
            composite(
                with_alpha(definition.seed.foreground, alpha),
                surfaces.surface,
            )
        };
        let (secondary_alpha, muted_alpha, decorative_alpha, disabled_alpha) =
            match definition.polarity {
                Polarity::Dark => (0.76, 0.58, 0.46, 0.36),
                Polarity::Light => (0.82, 0.64, 0.52, 0.42),
            };

        let interaction_foreground = require_foreground(
            content(ContentRole::Primary),
            definition.seed.foreground,
            &neutral_colors,
            "all opaque neutral surfaces",
            targets.normal_text,
        )?;
        let hover_alpha = if high_emphasis { 0.14 } else { 0.08 };
        let pressed_alpha = if high_emphasis { 0.22 } else { 0.14 };
        let hover = with_alpha(interaction_foreground, hover_alpha);
        let pressed = with_alpha(interaction_foreground, pressed_alpha);
        let interactive_neutral = interactive_backgrounds(&neutral, hover, pressed);
        let interactive_colors: Vec<Color> = interactive_neutral
            .iter()
            .map(|(_, color)| *color)
            .collect();

        let primary = require_foreground(
            content(ContentRole::Primary),
            definition.seed.foreground,
            &interactive_colors,
            "all supported neutral surface states",
            targets.normal_text,
        )?;
        let secondary = require_foreground(
            content(ContentRole::Secondary),
            faded(secondary_alpha),
            &interactive_colors,
            "all supported neutral surface states",
            targets.normal_text,
        )?;
        let muted = require_foreground(
            content(ContentRole::Muted),
            faded(muted_alpha),
            &interactive_colors,
            "all supported neutral surface states",
            targets.normal_text,
        )?;
        let decorative = optional_foreground(
            content(ContentRole::Decorative),
            faded(decorative_alpha),
            &interactive_colors,
            "all supported neutral surface states",
            targets.decorative,
        )?;
        let disabled = optional_foreground(
            content(ContentRole::Disabled),
            faded(disabled_alpha),
            &interactive_colors,
            "all supported neutral surface states",
            targets.decorative,
        )?;
        let inverse = require_foreground(
            content(ContentRole::Inverse),
            definition.seed.background,
            &[primary],
            "the resolved inverted neutral tone",
            targets.normal_text,
        )?;
        let content_tokens = Content {
            primary,
            secondary,
            muted,
            decorative,
            disabled,
            inverse,
        };

        let borders = Borders {
            subtle: faded(match definition.polarity {
                Polarity::Dark => 0.16,
                Polarity::Light => 0.18,
            }),
            standard: require_foreground(
                border(BorderRole::Standard),
                faded(match definition.polarity {
                    Polarity::Dark => 0.24,
                    Polarity::Light => 0.28,
                }),
                &neutral_colors,
                "all opaque neutral surfaces",
                1.5,
            )?,
            strong: require_foreground(
                border(BorderRole::Strong),
                faded(match definition.polarity {
                    Polarity::Dark => 0.36,
                    Polarity::Light => 0.42,
                }),
                &neutral_colors,
                "all opaque neutral surfaces",
                targets.boundary,
            )?,
            focus: require_foreground(
                border(BorderRole::Focus),
                definition.seed.accent,
                &neutral_colors,
                "all opaque neutral surfaces",
                targets.boundary,
            )?,
            selected: require_foreground(
                border(BorderRole::Selected),
                definition.seed.accent,
                &neutral_colors,
                "all opaque neutral surfaces",
                targets.boundary,
            )?,
        };

        let semantic_resolver = SemanticResolver::new(
            surfaces.surface,
            &neutral_colors,
            &interactive_colors,
            targets.normal_text,
            targets.boundary,
            high_emphasis,
        );

        let accent = semantic_resolver.resolve(Intent::Accent, definition.seed.accent)?;
        let success = semantic_resolver.resolve(Intent::Success, definition.seed.success)?;
        let warning = semantic_resolver.resolve(Intent::Warning, definition.seed.warning)?;
        let danger = semantic_resolver.resolve(Intent::Danger, definition.seed.danger)?;
        let info = semantic_resolver.resolve(Intent::Info, definition.seed.info)?;

        let interaction = Interaction {
            hover,
            pressed,
            selected: with_alpha(
                accent.solid.active.color,
                if high_emphasis { 0.26 } else { 0.16 },
            ),
            selected_hover: with_alpha(
                accent.solid.active.color,
                if high_emphasis { 0.34 } else { 0.24 },
            ),
            dragged: with_alpha(
                accent.solid.active.color,
                if high_emphasis { 0.38 } else { 0.30 },
            ),
            drop_target: with_alpha(
                info.solid.active.color,
                if high_emphasis { 0.32 } else { 0.22 },
            ),
        };

        let colors = Self {
            surfaces,
            content: content_tokens,
            borders,
            interaction,
            accent,
            success,
            warning,
            danger,
            info,
        };
        colors.validate(definition)?;

        Ok(colors)
    }

    fn validate(&self, definition: &Definition) -> Result<(), ResolveError> {
        let targets = definition.profile.targets();
        let neutral = neutral_backgrounds(&self.surfaces);
        let interactive =
            interactive_backgrounds(&neutral, self.interaction.hover, self.interaction.pressed);

        validate_contrast(
            content(ContentRole::Primary),
            self.content.primary,
            &interactive,
            targets.normal_text,
        )?;
        validate_contrast(
            content(ContentRole::Secondary),
            self.content.secondary,
            &interactive,
            targets.normal_text,
        )?;
        validate_contrast(
            content(ContentRole::Muted),
            self.content.muted,
            &interactive,
            targets.normal_text,
        )?;
        if targets.decorative >= 1.0 {
            validate_contrast(
                content(ContentRole::Decorative),
                self.content.decorative,
                &interactive,
                targets.decorative,
            )?;
            validate_contrast(
                content(ContentRole::Disabled),
                self.content.disabled,
                &interactive,
                targets.decorative,
            )?;
        }
        validate_contrast(
            content(ContentRole::Inverse),
            self.content.inverse,
            &[(
                "content.primary as an inverted neutral tone",
                self.content.primary,
            )],
            targets.normal_text,
        )?;
        validate_contrast(
            border(BorderRole::Strong),
            self.borders.strong,
            &neutral,
            targets.boundary,
        )?;
        validate_contrast(
            border(BorderRole::Focus),
            self.borders.focus,
            &neutral,
            targets.boundary,
        )?;
        validate_contrast(
            border(BorderRole::Selected),
            self.borders.selected,
            &neutral,
            targets.boundary,
        )?;

        for (intent, semantic) in [
            (Intent::Accent, self.accent),
            (Intent::Success, self.success),
            (Intent::Warning, self.warning),
            (Intent::Danger, self.danger),
            (Intent::Info, self.info),
        ] {
            super::semantic::validate(
                intent,
                semantic,
                &interactive,
                &neutral,
                targets.normal_text,
                targets.boundary,
            )?;
        }

        Ok(())
    }
}

fn validate_definition(definition: &Definition) -> Result<(), ResolveError> {
    let targets = definition.profile.targets();
    for (target, value, optional) in [
        ("normal_text", targets.normal_text, false),
        ("large_text", targets.large_text, false),
        ("boundary", targets.boundary, false),
        ("decorative", targets.decorative, true),
    ] {
        if !(1.0..=21.0).contains(&value) && !(optional && value == 0.0) {
            return Err(ResolveError::InvalidTarget { target, value });
        }
    }

    let seed = definition.seed;
    for (name, color) in [
        ("background", seed.background),
        ("foreground", seed.foreground),
        ("tint", seed.tint),
        ("shade", seed.shade),
        ("accent", seed.accent),
        ("success", seed.success),
        ("warning", seed.warning),
        ("danger", seed.danger),
        ("info", seed.info),
    ] {
        let alpha = color.components()[3];
        if (alpha - 1.0).abs() > f32::EPSILON {
            return Err(ResolveError::NonOpaqueSeed { seed: name, alpha });
        }
    }

    Ok(())
}

fn resolve_surfaces(definition: &Definition) -> Surfaces {
    let seed = definition.seed;
    let (inset, canvas, sunken, raised, overlay, field, scrim_alpha) = match definition.polarity {
        Polarity::Dark => (0.16, 0.32, 0.48, 0.05, 0.09, 0.07, 0.64),
        Polarity::Light => (0.04, 0.08, 0.12, 0.35, 0.65, 0.95, 0.38),
    };

    Surfaces {
        sunken: seed.background.mix_oklab(seed.shade, sunken),
        canvas: seed.background.mix_oklab(seed.shade, canvas),
        inset: seed.background.mix_oklab(seed.shade, inset),
        surface: seed.background,
        raised: seed.background.mix_oklab(seed.tint, raised),
        overlay: seed.background.mix_oklab(seed.tint, overlay),
        field: seed.background.mix_oklab(seed.tint, field),
        scrim: with_alpha(seed.shade, scrim_alpha),
    }
}

fn validate_surfaces(surfaces: &Surfaces, polarity: Polarity) -> Result<(), ResolveError> {
    let ordered = match polarity {
        Polarity::Dark => [
            ("surfaces.sunken", surfaces.sunken),
            ("surfaces.canvas", surfaces.canvas),
            ("surfaces.inset", surfaces.inset),
            ("surfaces.surface", surfaces.surface),
            ("surfaces.raised", surfaces.raised),
            ("surfaces.field", surfaces.field),
            ("surfaces.overlay", surfaces.overlay),
        ],
        Polarity::Light => neutral_backgrounds(surfaces),
    };
    for adjacent in ordered.windows(2) {
        let [(first_name, first), (second_name, second)] = adjacent else {
            unreachable!()
        };
        let first_lightness = Oklch::from(*first).components()[0];
        let second_lightness = Oklch::from(*second).components()[0];
        if first_lightness >= second_lightness {
            return Err(ResolveError::SurfaceOrder {
                darker: first_name,
                lighter: second_name,
            });
        }
        let difference = second_lightness - first_lightness;
        if difference < MINIMUM_SURFACE_LIGHTNESS_DELTA {
            return Err(ResolveError::SurfaceSeparation {
                first: first_name,
                second: second_name,
                difference,
                minimum: MINIMUM_SURFACE_LIGHTNESS_DELTA,
            });
        }
    }

    Ok(())
}

pub(super) fn require_foreground(
    token: TokenRole,
    candidate: Color,
    backgrounds: &[Color],
    against: &'static str,
    minimum_ratio: f32,
) -> Result<Color, ResolveError> {
    adjust_foreground(candidate, backgrounds, minimum_ratio).ok_or(
        ResolveError::UnsatisfiableContrast {
            token,
            against,
            minimum_ratio,
        },
    )
}

fn optional_foreground(
    token: TokenRole,
    candidate: Color,
    backgrounds: &[Color],
    against: &'static str,
    minimum_ratio: f32,
) -> Result<Color, ResolveError> {
    if minimum_ratio < 1.0 {
        Ok(candidate)
    } else {
        require_foreground(token, candidate, backgrounds, against, minimum_ratio)
    }
}

fn neutral_backgrounds(surfaces: &Surfaces) -> [(&'static str, Color); 7] {
    [
        ("surfaces.sunken", surfaces.sunken),
        ("surfaces.canvas", surfaces.canvas),
        ("surfaces.inset", surfaces.inset),
        ("surfaces.surface", surfaces.surface),
        ("surfaces.raised", surfaces.raised),
        ("surfaces.overlay", surfaces.overlay),
        ("surfaces.field", surfaces.field),
    ]
}

fn interactive_backgrounds(
    neutral: &[(&'static str, Color)],
    hover: Color,
    pressed: Color,
) -> Vec<(&'static str, Color)> {
    let mut backgrounds = Vec::with_capacity(neutral.len() * 3);
    for (name, color) in neutral {
        backgrounds.push((*name, *color));
        backgrounds.push((
            "neutral surface + interaction.hover",
            composite(hover, *color),
        ));
        backgrounds.push((
            "neutral surface + interaction.pressed",
            composite(pressed, *color),
        ));
    }
    backgrounds
}

pub(super) fn validate_contrast(
    token: TokenRole,
    foreground: Color,
    backgrounds: &[(&'static str, Color)],
    minimum_ratio: f32,
) -> Result<(), ResolveError> {
    for (background, color) in backgrounds {
        let actual_ratio = contrast_ratio(foreground, *color);
        if actual_ratio + CONTRAST_EPSILON < minimum_ratio {
            return Err(ResolveError::ContrastViolation {
                token,
                background,
                actual_ratio,
                minimum_ratio,
            });
        }
    }
    Ok(())
}

pub(super) fn with_alpha(color: Color, alpha: f32) -> Color {
    let [red, green, blue, _] = color.components();
    Color::from_rgba(red, green, blue, alpha)
}

const fn content(role: ContentRole) -> TokenRole {
    TokenRole::Content(role)
}

const fn border(role: BorderRole) -> TokenRole {
    TokenRole::Border(role)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Metadata, Profile, Seed, Targets, token::SemanticRole};

    fn definition(polarity: Polarity, profile: Profile) -> Definition {
        let (background, foreground, tint, shade) = match polarity {
            Polarity::Dark => (
                Color::from_rgb(0.098, 0.106, 0.125),
                Color::from_rgb(0.902, 0.882, 0.835),
                Color::from_rgb(0.961, 0.945, 0.910),
                Color::from_rgb(0.031, 0.035, 0.047),
            ),
            Polarity::Light => (
                Color::from_rgb(0.941, 0.945, 0.937),
                Color::from_rgb(0.188, 0.204, 0.239),
                Color::from_rgb(1.0, 1.0, 1.0),
                Color::from_rgb(0.588, 0.608, 0.639),
            ),
        };
        Definition::new(
            Metadata {
                name: "Test".into(),
                description: None,
            },
            polarity,
            profile,
            Seed {
                background,
                foreground,
                tint,
                shade,
                accent: Color::from_rgb(0.541, 0.714, 0.784),
                success: Color::from_rgb(0.561, 0.741, 0.525),
                warning: Color::from_rgb(0.875, 0.706, 0.404),
                danger: Color::from_rgb(0.875, 0.486, 0.525),
                info: Color::from_rgb(0.361, 0.761, 0.733),
            },
        )
    }

    #[test]
    fn resolves_and_validates_standard_dark_tokens() {
        Colors::resolve(&definition(Polarity::Dark, Profile::Standard)).unwrap();
    }

    #[test]
    fn resolves_and_validates_high_contrast_light_tokens() {
        Colors::resolve(&definition(Polarity::Light, Profile::High)).unwrap();
    }

    #[test]
    fn resolves_every_builtin_polarity_and_profile_combination() {
        for polarity in [Polarity::Dark, Polarity::Light] {
            for profile in [Profile::Standard, Profile::High] {
                Colors::resolve(&definition(polarity, profile)).unwrap();
            }
        }
    }

    #[test]
    fn rejects_surfaces_without_perceptible_separation() {
        let mut definition = definition(Polarity::Light, Profile::Standard);
        definition.seed.tint = Color::from_rgb(0.95, 0.954, 0.946);

        assert!(matches!(
            Colors::resolve(&definition),
            Err(ResolveError::SurfaceSeparation {
                first: "surfaces.surface",
                second: "surfaces.raised",
                minimum: MINIMUM_SURFACE_LIGHTNESS_DELTA,
                ..
            })
        ));
    }

    #[test]
    fn readable_content_passes_over_supported_interaction_states() {
        for polarity in [Polarity::Dark, Polarity::Light] {
            let definition = definition(polarity, Profile::Standard);
            let colors = Colors::resolve(&definition).unwrap();
            let neutral = neutral_backgrounds(&colors.surfaces);
            let interactive = interactive_backgrounds(
                &neutral,
                colors.interaction.hover,
                colors.interaction.pressed,
            );

            for (role, foreground) in [
                (ContentRole::Primary, colors.content.primary),
                (ContentRole::Secondary, colors.content.secondary),
                (ContentRole::Muted, colors.content.muted),
            ] {
                validate_contrast(
                    content(role),
                    foreground,
                    &interactive,
                    definition.profile.targets().normal_text,
                )
                .unwrap();
            }
        }
    }

    #[test]
    fn reports_the_token_for_an_unsatisfiable_contrast_contract() {
        let definition = definition(
            Polarity::Dark,
            Profile::Custom(Targets {
                normal_text: 21.0,
                large_text: 4.5,
                boundary: 4.5,
                decorative: 0.0,
            }),
        );

        let error = Colors::resolve(&definition).unwrap_err();
        assert!(matches!(
            error,
            ResolveError::UnsatisfiableContrast {
                token: TokenRole::Content(ContentRole::Primary),
                against: "all opaque neutral surfaces",
                minimum_ratio: 21.0,
            }
        ));
    }

    #[test]
    fn rejects_translucent_seeds_with_context() {
        let mut definition = definition(Polarity::Dark, Profile::Standard);
        definition.seed.warning = Color::from_rgba(0.8, 0.6, 0.2, 0.5);

        assert_eq!(
            Colors::resolve(&definition),
            Err(ResolveError::NonOpaqueSeed {
                seed: "warning",
                alpha: 0.5,
            })
        );
    }

    #[test]
    fn rejects_invalid_custom_targets_before_derivation() {
        let definition = definition(
            Polarity::Dark,
            Profile::Custom(Targets {
                normal_text: 22.0,
                large_text: 4.5,
                boundary: 3.0,
                decorative: 0.0,
            }),
        );

        assert_eq!(
            Colors::resolve(&definition),
            Err(ResolveError::InvalidTarget {
                target: "normal_text",
                value: 22.0,
            })
        );
    }

    #[test]
    fn final_validation_identifies_a_failing_semantic_state_pair() {
        let definition = definition(Polarity::Dark, Profile::Standard);
        let mut colors = Colors::resolve(&definition).unwrap();
        colors.danger.soft.hovered.text = colors.danger.soft.hovered.color;

        let error = colors.validate(&definition).unwrap_err();
        assert!(matches!(
            error,
            ResolveError::ContrastViolation {
                token: TokenRole::Semantic(Intent::Danger, SemanticRole::Soft),
                background: "hovered fill",
                ..
            }
        ));
    }
}
