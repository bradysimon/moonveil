use crate::{
    Color,
    color::Oklch,
    contrast::{adjust_semantic_solid, composite, on_color_foreground},
};

use super::{
    Fill, Pair, ResolveError, Semantic, SemanticIntent, SemanticRole, TokenRole,
    resolve::{require_foreground, validate_contrast, with_alpha},
};

pub(super) struct Resolver<'a> {
    surface: Color,
    neutral: &'a [Color],
    interactive_neutral: &'a [Color],
    text_target: f32,
    boundary_target: f32,
    high_emphasis: bool,
}

impl<'a> Resolver<'a> {
    pub(super) fn new(
        surface: Color,
        neutral: &'a [Color],
        interactive_neutral: &'a [Color],
        text_target: f32,
        boundary_target: f32,
        high_emphasis: bool,
    ) -> Self {
        Self {
            surface,
            neutral,
            interactive_neutral,
            text_target,
            boundary_target,
            high_emphasis,
        }
    }

    pub(super) fn resolve(
        &self,
        intent: SemanticIntent,
        seed: Color,
    ) -> Result<Semantic, ResolveError> {
        let solid_token = token(intent, SemanticRole::Solid);
        let solid_color = adjust_semantic_solid(seed, self.text_target).ok_or(
            ResolveError::UnsatisfiableContrast {
                token: solid_token,
                against: "deterministic on-color text",
                minimum_ratio: self.text_target,
            },
        )?;
        let solid =
            self.resolve_fill(solid_color, on_color_foreground(solid_color), solid_token)?;
        let soft_color = composite(
            with_alpha(solid_color, if self.high_emphasis { 0.22 } else { 0.15 }),
            self.surface,
        );
        let soft = self.resolve_fill(soft_color, solid_color, token(intent, SemanticRole::Soft))?;

        Ok(Semantic {
            foreground: require_foreground(
                token(intent, SemanticRole::Foreground),
                seed,
                self.interactive_neutral,
                "all supported neutral surface states",
                self.text_target,
            )?,
            indicator: require_foreground(
                token(intent, SemanticRole::Indicator),
                seed,
                self.neutral,
                "all opaque neutral surfaces and control tracks",
                self.boundary_target,
            )?,
            solid,
            soft,
            border: require_foreground(
                token(intent, SemanticRole::Border),
                seed,
                self.neutral,
                "all opaque neutral surfaces",
                self.boundary_target,
            )?,
        })
    }

    fn resolve_fill(
        &self,
        color: Color,
        text_candidate: Color,
        token: TokenRole,
    ) -> Result<Fill, ResolveError> {
        let text_lightness = Oklch::from(text_candidate).components()[0];
        let state_endpoint = Color::from(Oklch::new(
            if text_lightness > 0.5 { 0.03 } else { 0.99 },
            0.0,
            0.0,
            1.0,
        ));
        let hovered = composite(
            with_alpha(state_endpoint, if self.high_emphasis { 0.14 } else { 0.08 }),
            color,
        );
        let pressed = composite(
            with_alpha(state_endpoint, if self.high_emphasis { 0.22 } else { 0.14 }),
            color,
        );
        let text = require_foreground(
            token,
            text_candidate,
            &[color, hovered, pressed],
            "the active, hovered, and pressed fills",
            self.text_target,
        )?;

        Ok(Fill {
            active: Pair { color, text },
            hovered: Pair {
                color: hovered,
                text,
            },
            pressed: Pair {
                color: pressed,
                text,
            },
        })
    }
}

pub(super) fn validate(
    intent: SemanticIntent,
    semantic: Semantic,
    interactive: &[(&'static str, Color)],
    neutral: &[(&'static str, Color)],
    text_target: f32,
    boundary_target: f32,
) -> Result<(), ResolveError> {
    validate_contrast(
        token(intent, SemanticRole::Foreground),
        semantic.foreground,
        interactive,
        text_target,
    )?;
    validate_contrast(
        token(intent, SemanticRole::Indicator),
        semantic.indicator,
        neutral,
        boundary_target,
    )?;
    validate_contrast(
        token(intent, SemanticRole::Border),
        semantic.border,
        neutral,
        boundary_target,
    )?;
    validate_fill(
        semantic.solid,
        token(intent, SemanticRole::Solid),
        text_target,
    )?;
    validate_fill(
        semantic.soft,
        token(intent, SemanticRole::Soft),
        text_target,
    )
}

fn validate_fill(fill: Fill, token: TokenRole, minimum_ratio: f32) -> Result<(), ResolveError> {
    for (background, pair) in [
        ("active fill", fill.active),
        ("hovered fill", fill.hovered),
        ("pressed fill", fill.pressed),
    ] {
        validate_contrast(token, pair.text, &[(background, pair.color)], minimum_ratio)?;
    }
    Ok(())
}

const fn token(intent: SemanticIntent, role: SemanticRole) -> TokenRole {
    TokenRole::Semantic(intent, role)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_adjustment_does_not_shift_standalone_semantic_roles() {
        let surface = Color::from_rgb(0.0, 0.0, 0.0);
        let seed = Color::from_rgb(0.5, 0.5, 0.5);
        let backgrounds = [surface];
        let resolver = Resolver::new(surface, &backgrounds, &backgrounds, 7.0, 3.0, true);

        let semantic = resolver.resolve(SemanticIntent::Accent, seed).unwrap();

        assert_eq!(semantic.indicator, seed);
        assert_eq!(semantic.border, seed);
    }
}
