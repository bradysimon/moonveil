//! Color compositing and WCAG contrast measurement.

use crate::color::{Color, Oklch};

const SEARCH_ITERATIONS: usize = 24;
const DARK_ON_COLOR_LIGHTNESS: f32 = 0.03;
const LIGHT_ON_COLOR_LIGHTNESS: f32 = 0.99;

/// The contrast policy used to resolve a theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Contrast {
    /// Standard WCAG targets.
    Standard,
    /// Stricter high-contrast targets.
    High,
    /// User-supplied contrast targets.
    Custom(Targets),
}

impl Contrast {
    /// Returns the concrete targets for this profile.
    pub const fn targets(self) -> Targets {
        match self {
            Self::Standard => Targets::STANDARD,
            Self::High => Targets::HIGH,
            Self::Custom(targets) => targets,
        }
    }
}

/// Contrast ratios used while resolving accessible tokens.
///
/// Higher contrast ratios are more accessible and may be required by some users,
/// but limit the range of colors that can be used.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Targets {
    /// Minimum ratio for normal-size text.
    pub normal_text: f32,
    /// Minimum ratio for large text.
    pub large_text: f32,
    /// Minimum ratio for essential boundaries and focus indicators.
    pub boundary: f32,
    /// Minimum ratio for disabled or decorative content.
    pub decorative: f32,
}

impl Targets {
    /// WCAG targets used by standard apps.
    pub const STANDARD: Self = Self {
        normal_text: 4.5,
        large_text: 3.0,
        boundary: 3.0,
        decorative: 0.0,
    };

    /// Stricter targets used by high-contrast apps.
    pub const HIGH: Self = Self {
        normal_text: 7.0,
        large_text: 4.5,
        boundary: 4.5,
        decorative: 3.0,
    };
}

/// Composites `foreground` over `background` using source-over alpha blending.
///
/// Compositing resolves transparency to the color a user actually sees. For
/// example, partially transparent black text on a white surface is displayed as
/// gray, so measuring the original black would overstate its contrast. When the
/// background is opaque, the returned color is also opaque and can be measured
/// directly for contrast.
pub(crate) fn composite(foreground: Color, background: Color) -> Color {
    let [f_red, f_green, f_blue, f_alpha] = foreground.components();
    let [b_red, b_green, b_blue, b_alpha] = background.components();

    let alpha = f_alpha + b_alpha * (1.0 - f_alpha);

    if alpha <= f32::EPSILON {
        return Color::new(0.0, 0.0, 0.0, 0.0);
    }

    let blend = |f_channel: f32, b_channel: f32| {
        (f_channel * f_alpha + b_channel * b_alpha * (1.0 - f_alpha)) / alpha
    };

    Color::new(
        blend(f_red, b_red),
        blend(f_green, b_green),
        blend(f_blue, b_blue),
        alpha,
    )
}

/// Returns the WCAG 2.2 contrast ratio after compositing `foreground` over an
/// opaque `background`.
///
/// Ratios range from `1.0` for identical colors to `21.0` for black and white.
/// We target at least `4.5:1` for normal text and `3:1` for large text and
/// important component boundaries. High contrast profiles should raise those
/// targets to `7:1` and `4.5:1` respectively.
pub(crate) fn contrast_ratio(foreground: Color, background: Color) -> f32 {
    let [.., background_alpha] = background.components();
    debug_assert!(
        (background_alpha - 1.0).abs() <= f32::EPSILON,
        "contrast requires an opaque background"
    );

    let f_luminance = relative_luminance(composite(foreground, background));
    let b_luminance = relative_luminance(background);
    let lighter = f_luminance.max(b_luminance);
    let darker = f_luminance.min(b_luminance);

    (lighter + 0.05) / (darker + 0.05)
}

/// Adjusts an opaque foreground to meet `minimum_ratio` against every background.
///
/// The original color is returned unchanged when it already passes. Otherwise,
/// this finds every relative-luminance interval that satisfies all backgrounds
/// and returns the passing color with the smallest Oklch lightness change. Hue
/// and alpha are preserved, and chroma is only reduced by sRGB gamut mapping.
/// Returns `None` when neither direction can satisfy the requested ratio.
pub(crate) fn adjust_foreground(
    foreground: Color,
    backgrounds: &[Color],
    minimum_ratio: f32,
) -> Option<Color> {
    debug_assert!(
        (1.0..=21.0).contains(&minimum_ratio),
        "contrast ratio must be in the range [1.0, 21.0]"
    );
    let [lightness, chroma, hue, alpha] = Oklch::from(foreground).components();
    debug_assert!(
        (alpha - 1.0).abs() <= f32::EPSILON,
        "adjusted foregrounds must be opaque"
    );

    if meets_contrast(foreground, backgrounds, minimum_ratio) {
        return Some(foreground);
    }

    passing_luminance_intervals(backgrounds, minimum_ratio)
        .into_iter()
        .filter_map(|interval| {
            let [minimum, maximum] = inset_interval(interval);
            let minimum_lightness =
                lightness_for_luminance(minimum, chroma, hue, alpha, LuminanceBound::Minimum);
            let maximum_lightness =
                lightness_for_luminance(maximum, chroma, hue, alpha, LuminanceBound::Maximum);
            let candidate = Color::from(Oklch::new(
                lightness.clamp(minimum_lightness, maximum_lightness),
                chroma,
                hue,
                alpha,
            ));

            meets_contrast(candidate, backgrounds, minimum_ratio).then_some(candidate)
        })
        .min_by(|first, second| {
            let first_delta = (Oklch::from(*first).components()[0] - lightness).abs();
            let second_delta = (Oklch::from(*second).components()[0] - lightness).abs();
            first_delta.total_cmp(&second_delta)
        })
}

/// Selects deterministic near-black or near-white text for an opaque fill.
///
/// The on-color candidate with the greater contrast ratio is returned. Using
/// fixed, achromatic Oklch tones keeps semantic fill pairs stable across themes
/// and avoids introducing a second hue into their text color.
pub(crate) fn on_color_foreground(background: Color) -> Color {
    let dark = on_color(DARK_ON_COLOR_LIGHTNESS);
    let light = on_color(LIGHT_ON_COLOR_LIGHTNESS);

    if contrast_ratio(dark, background) >= contrast_ratio(light, background) {
        dark
    } else {
        light
    }
}

/// Adjusts an opaque semantic solid until it supports on-color foreground text.
///
/// The solid is returned unchanged when either on-color candidate already
/// meets `minimum_ratio`. Otherwise, this searches toward both ends of the
/// solid's Oklch lightness range and returns the nearest passing tone. Hue is
/// preserved and chroma is only reduced when required by sRGB gamut mapping.
/// Returns `None` if the requested ratio cannot be met with either candidate.
pub(crate) fn adjust_semantic_solid(solid: Color, minimum_ratio: f32) -> Option<Color> {
    let [.., alpha] = solid.components();
    debug_assert!(
        (alpha - 1.0).abs() <= f32::EPSILON,
        "semantic solids must be opaque"
    );
    debug_assert!(
        (1.0..=21.0).contains(&minimum_ratio),
        "contrast ratio must be in the range [1.0, 21.0]"
    );

    if maximum_on_color_contrast(solid) >= minimum_ratio {
        return Some(solid);
    }

    let [lightness, chroma, hue, alpha] = Oklch::from(solid).components();
    let darker = search_solid_lightness([lightness, 0.0], chroma, hue, alpha, minimum_ratio);
    let lighter = search_solid_lightness([lightness, 1.0], chroma, hue, alpha, minimum_ratio);

    match (darker, lighter) {
        (Some(darker), Some(lighter)) => {
            let darker_delta = lightness - Oklch::from(darker).components()[0];
            let lighter_delta = Oklch::from(lighter).components()[0] - lightness;

            Some(if darker_delta <= lighter_delta {
                darker
            } else {
                lighter
            })
        }
        (Some(darker), None) => Some(darker),
        (None, Some(lighter)) => Some(lighter),
        (None, None) => None,
    }
}

fn passing_luminance_intervals(backgrounds: &[Color], minimum_ratio: f32) -> Vec<[f32; 2]> {
    let mut intervals: Vec<[f32; 2]> = vec![[0.0, 1.0]];

    for background in backgrounds {
        let luminance = relative_luminance(*background);
        let darker_maximum = (luminance + 0.05) / minimum_ratio - 0.05;
        let lighter_minimum = minimum_ratio * (luminance + 0.05) - 0.05;
        let mut allowed = Vec::with_capacity(2);

        if darker_maximum >= 0.0 {
            allowed.push([0.0, darker_maximum.min(1.0)]);
        }
        if lighter_minimum <= 1.0 {
            allowed.push([lighter_minimum.max(0.0), 1.0]);
        }

        intervals = intervals
            .into_iter()
            .flat_map(|[start, end]| {
                allowed
                    .iter()
                    .filter_map(move |[allowed_start, allowed_end]| {
                        let intersection = [start.max(*allowed_start), end.min(*allowed_end)];
                        (intersection[0] <= intersection[1]).then_some(intersection)
                    })
            })
            .collect();

        if intervals.is_empty() {
            break;
        }
    }

    intervals
}

fn inset_interval([start, end]: [f32; 2]) -> [f32; 2] {
    let inset = ((end - start) / 4.0).min(0.000_01);
    [start + inset, end - inset]
}

enum LuminanceBound {
    Minimum,
    Maximum,
}

fn lightness_for_luminance(
    target: f32,
    chroma: f32,
    hue: f32,
    alpha: f32,
    bound: LuminanceBound,
) -> f32 {
    let (mut lower, mut upper) = (0.0, 1.0);

    for _ in 0..SEARCH_ITERATIONS {
        let lightness = (lower + upper) / 2.0;
        let candidate = Color::from(Oklch::new(lightness, chroma, hue, alpha));

        if relative_luminance(candidate) < target {
            lower = lightness;
        } else {
            upper = lightness;
        }
    }

    match bound {
        LuminanceBound::Minimum => upper,
        LuminanceBound::Maximum => lower,
    }
}

fn search_solid_lightness(
    [start, end]: [f32; 2],
    chroma: f32,
    hue: f32,
    alpha: f32,
    minimum_ratio: f32,
) -> Option<Color> {
    let endpoint = Color::from(Oklch::new(end, chroma, hue, alpha));

    if maximum_on_color_contrast(endpoint) < minimum_ratio {
        return None;
    }

    let (mut failing, mut passing) = (start, end);

    for _ in 0..SEARCH_ITERATIONS {
        let lightness = (failing + passing) / 2.0;
        let candidate = Color::from(Oklch::new(lightness, chroma, hue, alpha));

        if maximum_on_color_contrast(candidate) >= minimum_ratio {
            passing = lightness;
        } else {
            failing = lightness;
        }
    }

    Some(Color::from(Oklch::new(passing, chroma, hue, alpha)))
}

fn maximum_on_color_contrast(background: Color) -> f32 {
    contrast_ratio(on_color_foreground(background), background)
}

fn on_color(lightness: f32) -> Color {
    Color::from(Oklch::new(lightness, 0.0, 0.0, 1.0))
}

/// Whether the given `foreground` meets `minimum_ratio` against every value in `backgrounds`.
fn meets_contrast(foreground: Color, backgrounds: &[Color], minimum_ratio: f32) -> bool {
    backgrounds
        .iter()
        .all(|background| contrast_ratio(foreground, *background) >= minimum_ratio)
}

/// Returns the WCAG relative luminance of an opaque sRGB color.
fn relative_luminance(color: Color) -> f32 {
    let [red, green, blue, alpha] = color.components();
    debug_assert!(
        (alpha - 1.0).abs() <= f32::EPSILON,
        "relative luminance requires an opaque color"
    );

    0.2126 * linearize(red) + 0.7152 * linearize(green) + 0.0722 * linearize(blue)
}

/// Converts a gamma-encoded sRGB channel to its linear-light value.
fn linearize(channel: f32) -> f32 {
    if channel <= 0.040_45 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const EPSILON: f32 = 0.000_01;
    const HUE_ERROR_EPSILON: f32 = 0.000_01;
    const PROPERTY_EPSILON: f32 = 0.000_2;

    fn assert_approx_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= EPSILON,
            "expected {expected}, got {actual}"
        );
    }

    fn assert_color_approx_eq(actual: Color, expected: Color) {
        for (actual, expected) in actual.components().into_iter().zip(expected.components()) {
            assert_approx_eq(actual, expected);
        }
    }

    fn color() -> impl Strategy<Value = Color> {
        (0.0f32..1.0, 0.0f32..1.0, 0.0f32..1.0, 0.0f32..1.0)
            .prop_map(|(red, green, blue, alpha)| Color::new(red, green, blue, alpha))
    }

    fn visible_color() -> impl Strategy<Value = Color> {
        (0.0f32..1.0, 0.0f32..1.0, 0.0f32..1.0, 0.001f32..1.0)
            .prop_map(|(red, green, blue, alpha)| Color::new(red, green, blue, alpha))
    }

    fn opaque_color() -> impl Strategy<Value = Color> {
        (0.0f32..1.0, 0.0f32..1.0, 0.0f32..1.0)
            .prop_map(|(red, green, blue)| Color::new(red, green, blue, 1.0))
    }

    fn opaque_backgrounds() -> impl Strategy<Value = Vec<Color>> {
        prop::collection::vec(opaque_color(), 1..5)
    }

    fn circular_hue_difference(first: f32, second: f32) -> f32 {
        let difference = (first - second).abs();
        difference.min(360.0 - difference)
    }

    fn oklab_hue_error(chroma: f32, first: f32, second: f32) -> f32 {
        let radians = circular_hue_difference(first, second).to_radians();
        2.0 * chroma * (radians / 2.0).sin()
    }

    fn components_are_valid(color: Color) -> bool {
        color
            .components()
            .into_iter()
            .all(|component| component.is_finite() && (0.0..=1.0).contains(&component))
    }

    #[test]
    fn composites_translucent_foreground_over_opaque_background() {
        let actual = composite(
            Color::new(1.0, 0.0, 0.0, 0.5),
            Color::new(0.0, 0.0, 1.0, 1.0),
        );
        assert_color_approx_eq(actual, Color::new(0.5, 0.0, 0.5, 1.0));
    }

    #[test]
    fn composites_two_translucent_colors() {
        let actual = composite(
            Color::new(1.0, 0.0, 0.0, 0.5),
            Color::new(0.0, 0.0, 1.0, 0.5),
        );
        assert_color_approx_eq(actual, Color::new(2.0 / 3.0, 0.0, 1.0 / 3.0, 0.75));
    }

    #[test]
    fn black_and_white_have_maximum_contrast() {
        let ratio = contrast_ratio(
            Color::new(0.0, 0.0, 0.0, 1.0),
            Color::new(1.0, 1.0, 1.0, 1.0),
        );
        assert_approx_eq(ratio, 21.0);
    }

    #[test]
    fn equal_colors_have_minimum_contrast() {
        let color = Color::new(0.25, 0.5, 0.75, 1.0);
        assert_approx_eq(contrast_ratio(color, color), 1.0);
    }

    #[test]
    fn contrast_composites_alpha_before_measuring() {
        let ratio = contrast_ratio(
            Color::new(0.0, 0.0, 0.0, 0.5),
            Color::new(1.0, 1.0, 1.0, 1.0),
        );
        assert_approx_eq(ratio, 3.976_653);
    }

    #[test]
    fn adjustment_leaves_a_passing_foreground_unchanged() {
        let foreground = Color::new(0.0, 0.0, 0.0, 1.0);
        let background = Color::new(1.0, 1.0, 1.0, 1.0);

        assert_eq!(
            adjust_foreground(foreground, &[background], 4.5),
            Some(foreground)
        );
    }

    #[test]
    fn adjustment_darkens_foreground_on_a_light_background() {
        let foreground = Color::new(0.8, 0.5, 0.4, 1.0);
        let background = Color::new(1.0, 1.0, 1.0, 1.0);
        let adjusted = adjust_foreground(foreground, &[background], 4.5).unwrap();
        let [original_lightness, original_chroma, original_hue, ..] =
            Oklch::from(foreground).components();
        let [adjusted_lightness, adjusted_chroma, adjusted_hue, ..] =
            Oklch::from(adjusted).components();

        assert!(adjusted_lightness < original_lightness);
        assert!(adjusted_chroma <= original_chroma + EPSILON);
        assert!((adjusted_hue - original_hue).abs() <= 0.001);
        assert!(contrast_ratio(adjusted, background) >= 4.5);
    }

    #[test]
    fn adjustment_lightens_foreground_on_a_dark_background() {
        let foreground = Color::new(0.2, 0.3, 0.7, 1.0);
        let background = Color::new(0.0, 0.0, 0.0, 1.0);
        let adjusted = adjust_foreground(foreground, &[background], 4.5).unwrap();
        let [original_lightness, ..] = Oklch::from(foreground).components();
        let [adjusted_lightness, ..] = Oklch::from(adjusted).components();

        assert!(adjusted_lightness > original_lightness);
        assert!(contrast_ratio(adjusted, background) >= 4.5);
    }

    #[test]
    fn adjustment_returns_none_when_no_direction_can_pass_every_background() {
        let foreground = Color::new(0.5, 0.5, 0.5, 1.0);
        let backgrounds = [
            Color::new(0.0, 0.0, 0.0, 1.0),
            Color::new(1.0, 1.0, 1.0, 1.0),
        ];

        assert_eq!(adjust_foreground(foreground, &backgrounds, 7.0), None);
    }

    #[test]
    fn adjustment_finds_a_passing_tone_when_both_endpoints_fail() {
        let foreground = Color::new(0.5, 0.5, 0.5, 1.0);
        let backgrounds = [
            Color::new(0.0, 0.0, 0.0, 1.0),
            Color::new(1.0, 1.0, 1.0, 1.0),
        ];

        let adjusted = adjust_foreground(foreground, &backgrounds, 4.5).unwrap();

        assert!(
            backgrounds
                .into_iter()
                .all(|background| contrast_ratio(adjusted, background) >= 4.5)
        );
    }

    #[test]
    fn foreground_adjustment_preserves_hue_after_gamut_mapping() {
        let foreground = Color::new(0.0, 0.649_028_9, 0.875_334_26, 1.0);
        let background = Color::new(0.835_592_4, 0.241_298_7, 0.227_970_02, 1.0);
        let adjusted = adjust_foreground(foreground, &[background], 4.581_033_7).unwrap();
        let original = Oklch::from(foreground);
        let adjusted = Oklch::from(adjusted);
        let original_hue = original.components()[2];
        let adjusted_hue = adjusted.components()[2];
        let hue_error = oklab_hue_error(adjusted.components()[1], adjusted_hue, original_hue);

        assert!(
            hue_error <= HUE_ERROR_EPSILON,
            "expected Oklab hue error <= {HUE_ERROR_EPSILON}, got {hue_error} ({original:?} -> {adjusted:?})"
        );
    }

    #[test]
    fn on_color_foreground_selects_near_black_for_a_light_fill() {
        let fill = Color::new(1.0, 1.0, 1.0, 1.0);

        assert_color_approx_eq(on_color_foreground(fill), on_color(DARK_ON_COLOR_LIGHTNESS));
    }

    #[test]
    fn on_color_foreground_selects_near_white_for_a_dark_fill() {
        let fill = Color::new(0.0, 0.0, 0.0, 1.0);

        assert_color_approx_eq(
            on_color_foreground(fill),
            on_color(LIGHT_ON_COLOR_LIGHTNESS),
        );
    }

    #[test]
    fn semantic_solid_is_unchanged_when_it_supports_on_color_text() {
        let solid = Color::new(0.5, 0.5, 0.5, 1.0);

        assert_eq!(adjust_semantic_solid(solid, 4.5), Some(solid));
    }

    #[test]
    fn semantic_solid_moves_to_the_nearest_tone_that_supports_text() {
        let solid = Color::new(0.5, 0.5, 0.5, 1.0);
        let adjusted = adjust_semantic_solid(solid, 7.0).unwrap();
        let [lightness, chroma, hue, alpha] = Oklch::from(solid).components();
        let darker = search_solid_lightness([lightness, 0.0], chroma, hue, alpha, 7.0).unwrap();
        let lighter = search_solid_lightness([lightness, 1.0], chroma, hue, alpha, 7.0).unwrap();
        let darker_delta = lightness - Oklch::from(darker).components()[0];
        let lighter_delta = Oklch::from(lighter).components()[0] - lightness;
        let expected = if darker_delta <= lighter_delta {
            darker
        } else {
            lighter
        };
        let text = on_color_foreground(adjusted);

        assert_color_approx_eq(adjusted, expected);
        assert!(contrast_ratio(text, adjusted) >= 7.0);
    }

    #[test]
    fn semantic_solid_returns_none_when_on_colors_cannot_meet_the_target() {
        let solid = Color::new(0.5, 0.5, 0.5, 1.0);

        assert_eq!(adjust_semantic_solid(solid, 21.0), None);
    }

    proptest! {
        #[test]
        fn transparent_foreground_preserves_visible_background(
            foreground in opaque_color(),
            background in visible_color(),
        ) {
            let [red, green, blue, ..] = foreground.components();
            let foreground = Color::new(red, green, blue, 0.0);
            let result = composite(foreground, background);

            for (actual, expected) in result.components().into_iter().zip(background.components()) {
                prop_assert!((actual - expected).abs() <= PROPERTY_EPSILON);
            }
        }

        #[test]
        fn opaque_foreground_replaces_background(foreground in opaque_color(), background in color()) {
            let result = composite(foreground, background);

            prop_assert_eq!(result, foreground);
        }

        #[test]
        fn compositing_arbitrary_colors_produces_bounded_components(
            foreground in color(),
            background in color(),
        ) {
            prop_assert!(components_are_valid(composite(foreground, background)));
        }

        #[test]
        fn compositing_over_opaque_background_is_opaque_and_bounded(
            foreground in color(),
            background in opaque_color(),
        ) {
            let result = composite(foreground, background);

            prop_assert!((result.components()[3] - 1.0).abs() <= PROPERTY_EPSILON);
            prop_assert!(components_are_valid(result));
        }

        #[test]
        fn opaque_contrast_is_symmetric(first in opaque_color(), second in opaque_color()) {
            let forward = contrast_ratio(first, second);
            let reverse = contrast_ratio(second, first);

            prop_assert!((forward - reverse).abs() <= PROPERTY_EPSILON);
        }

        #[test]
        fn contrast_ratio_is_bounded(foreground in color(), background in opaque_color()) {
            let ratio = contrast_ratio(foreground, background);

            prop_assert!(ratio.is_finite());
            prop_assert!((1.0..=21.0).contains(&ratio));
        }

        #[test]
        fn opaque_color_has_unit_contrast_with_itself(value in opaque_color()) {
            prop_assert!((contrast_ratio(value, value) - 1.0).abs() <= PROPERTY_EPSILON);
        }

        #[test]
        fn foreground_adjustment_satisfies_its_postconditions(
            foreground in opaque_color(),
            backgrounds in opaque_backgrounds(),
            minimum_ratio in 1.0f32..7.0,
        ) {
            if let Some(adjusted) = adjust_foreground(foreground, &backgrounds, minimum_ratio) {
                let [_, original_chroma, original_hue, original_alpha] =
                    Oklch::from(foreground).components();
                let [adjusted_lightness, adjusted_chroma, adjusted_hue, adjusted_alpha] =
                    Oklch::from(adjusted).components();

                prop_assert!(backgrounds.iter().all(|background|
                    contrast_ratio(adjusted, *background) + PROPERTY_EPSILON >= minimum_ratio
                ));
                prop_assert!((adjusted_alpha - original_alpha).abs() <= PROPERTY_EPSILON);
                prop_assert!(adjusted_chroma <= original_chroma + PROPERTY_EPSILON);
                prop_assert!(oklab_hue_error(adjusted_chroma, adjusted_hue, original_hue) <= HUE_ERROR_EPSILON);
                prop_assert!((0.0..=1.0).contains(&adjusted_lightness));
                prop_assert_eq!(adjust_foreground(adjusted, &backgrounds, minimum_ratio), Some(adjusted));
            }
        }

        #[test]
        fn passing_foreground_is_not_adjusted(
            foreground in opaque_color(),
            backgrounds in opaque_backgrounds(),
            fraction in 0.0f32..1.0,
        ) {
            let available_ratio = backgrounds
                .iter()
                .map(|background| contrast_ratio(foreground, *background))
                .fold(f32::INFINITY, f32::min);
            let minimum_ratio = 1.0 + (available_ratio - 1.0) * fraction;

            prop_assert_eq!(
                adjust_foreground(foreground, &backgrounds, minimum_ratio),
                Some(foreground)
            );
        }

        #[test]
        fn on_color_selects_the_higher_contrast_candidate(background in opaque_color()) {
            let selected = on_color_foreground(background);
            let dark_ratio = contrast_ratio(on_color(DARK_ON_COLOR_LIGHTNESS), background);
            let light_ratio = contrast_ratio(on_color(LIGHT_ON_COLOR_LIGHTNESS), background);

            prop_assert!(contrast_ratio(selected, background) + PROPERTY_EPSILON >= dark_ratio);
            prop_assert!(contrast_ratio(selected, background) + PROPERTY_EPSILON >= light_ratio);
        }

        #[test]
        fn semantic_solid_adjustment_satisfies_its_postconditions(
            solid in opaque_color(),
            minimum_ratio in 1.0f32..10.0,
        ) {
            let original_contrast = maximum_on_color_contrast(solid);

            if let Some(adjusted) = adjust_semantic_solid(solid, minimum_ratio) {
                let [_, original_chroma, original_hue, original_alpha] =
                    Oklch::from(solid).components();
                let [_, adjusted_chroma, adjusted_hue, adjusted_alpha] =
                    Oklch::from(adjusted).components();

                prop_assert!(
                    maximum_on_color_contrast(adjusted) + PROPERTY_EPSILON >= minimum_ratio
                );
                prop_assert!((adjusted_alpha - original_alpha).abs() <= PROPERTY_EPSILON);
                prop_assert!(adjusted_chroma <= original_chroma + PROPERTY_EPSILON);
                prop_assert!(oklab_hue_error(adjusted_chroma, adjusted_hue, original_hue) <= HUE_ERROR_EPSILON);
                if original_contrast >= minimum_ratio {
                    prop_assert_eq!(adjusted, solid);
                }
            }
        }
    }
}
