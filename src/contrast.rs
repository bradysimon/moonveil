//! Color compositing and WCAG contrast measurement.

use crate::color::{Oklch, Srgb};

const SEARCH_ITERATIONS: usize = 24;

/// Composites `foreground` over `background` using source-over alpha blending.
///
/// Compositing resolves transparency to the color a user actually sees. For
/// example, partially transparent black text on a white surface is displayed as
/// gray, so measuring the original black would overstate its contrast. When the
/// background is opaque, the returned color is also opaque and can be measured
/// directly for contrast.
pub(crate) fn composite(foreground: Srgb, background: Srgb) -> Srgb {
    let [f_red, f_green, f_blue, f_alpha] = foreground.components();
    let [b_red, b_green, b_blue, b_alpha] = background.components();

    let alpha = f_alpha + b_alpha * (1.0 - f_alpha);

    if alpha <= f32::EPSILON {
        return Srgb::new(0.0, 0.0, 0.0, 0.0);
    }

    let blend = |f_channel: f32, b_channel: f32| {
        (f_channel * f_alpha + b_channel * b_alpha * (1.0 - f_alpha)) / alpha
    };

    Srgb::new(
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
pub(crate) fn contrast_ratio(foreground: Srgb, background: Srgb) -> f32 {
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

/// Adjusts a foreground to meet `minimum_ratio` against every background.
///
/// The original color is returned unchanged when it already passes. Otherwise,
/// this searches toward both black and white in Oklch and returns the passing
/// color with the smallest lightness change. Hue and alpha are preserved;
/// chroma is only reduced by sRGB gamut mapping. Returns `None` when neither
/// direction can satisfy the requested ratio.
pub(crate) fn adjust_foreground(
    foreground: Srgb,
    backgrounds: &[Srgb],
    minimum_ratio: f32,
) -> Option<Srgb> {
    debug_assert!(
        (1.0..=21.0).contains(&minimum_ratio),
        "contrast ratio must be in the range [1.0, 21.0]"
    );

    if meets_contrast(foreground, backgrounds, minimum_ratio) {
        return Some(foreground);
    }

    let [lightness, chroma, hue, alpha] = Oklch::from(foreground).components();
    let darker = search_lightness(
        [lightness, 0.0],
        chroma,
        hue,
        alpha,
        backgrounds,
        minimum_ratio,
    );
    let lighter = search_lightness(
        [lightness, 1.0],
        chroma,
        hue,
        alpha,
        backgrounds,
        minimum_ratio,
    );

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

/// Searches from a failing Oklch `start` lightness toward a terminal `end`.
///
/// The endpoint is checked first because it establishes the passing side of the
/// binary-search interval. If it cannot meet `minimum_ratio` against every
/// background, no candidate in that direction is returned. Otherwise, the
/// search narrows the interval and returns the nearest known passing lightness.
/// Chroma, hue, and alpha remain fixed, except for any chroma reduction required
/// when the candidate is mapped into the sRGB gamut.
fn search_lightness(
    [start, end]: [f32; 2],
    chroma: f32,
    hue: f32,
    alpha: f32,
    backgrounds: &[Srgb],
    minimum_ratio: f32,
) -> Option<Srgb> {
    let endpoint = Srgb::from(Oklch::new(end, chroma, hue, alpha));

    if !meets_contrast(endpoint, backgrounds, minimum_ratio) {
        return None;
    }

    let (mut failing, mut passing) = (start, end);

    for _ in 0..SEARCH_ITERATIONS {
        let lightness = (failing + passing) / 2.0;
        let candidate = Srgb::from(Oklch::new(lightness, chroma, hue, alpha));

        if meets_contrast(candidate, backgrounds, minimum_ratio) {
            passing = lightness;
        } else {
            failing = lightness;
        }
    }

    Some(Srgb::from(Oklch::new(passing, chroma, hue, alpha)))
}

/// Whether the given `foreground` meets `minimum_ratio` against every value in `backgrounds`.
fn meets_contrast(foreground: Srgb, backgrounds: &[Srgb], minimum_ratio: f32) -> bool {
    backgrounds
        .iter()
        .all(|background| contrast_ratio(foreground, *background) >= minimum_ratio)
}

/// Returns the WCAG relative luminance of an opaque sRGB color.
fn relative_luminance(color: Srgb) -> f32 {
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

    const EPSILON: f32 = 0.000_01;

    fn assert_approx_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= EPSILON,
            "expected {expected}, got {actual}"
        );
    }

    fn assert_color_approx_eq(actual: Srgb, expected: Srgb) {
        for (actual, expected) in actual.components().into_iter().zip(expected.components()) {
            assert_approx_eq(actual, expected);
        }
    }

    #[test]
    fn composites_translucent_foreground_over_opaque_background() {
        let actual = composite(Srgb::new(1.0, 0.0, 0.0, 0.5), Srgb::new(0.0, 0.0, 1.0, 1.0));
        assert_color_approx_eq(actual, Srgb::new(0.5, 0.0, 0.5, 1.0));
    }

    #[test]
    fn composites_two_translucent_colors() {
        let actual = composite(Srgb::new(1.0, 0.0, 0.0, 0.5), Srgb::new(0.0, 0.0, 1.0, 0.5));
        assert_color_approx_eq(actual, Srgb::new(2.0 / 3.0, 0.0, 1.0 / 3.0, 0.75));
    }

    #[test]
    fn black_and_white_have_maximum_contrast() {
        let ratio = contrast_ratio(Srgb::new(0.0, 0.0, 0.0, 1.0), Srgb::new(1.0, 1.0, 1.0, 1.0));
        assert_approx_eq(ratio, 21.0);
    }

    #[test]
    fn equal_colors_have_minimum_contrast() {
        let color = Srgb::new(0.25, 0.5, 0.75, 1.0);
        assert_approx_eq(contrast_ratio(color, color), 1.0);
    }

    #[test]
    fn contrast_composites_alpha_before_measuring() {
        let ratio = contrast_ratio(Srgb::new(0.0, 0.0, 0.0, 0.5), Srgb::new(1.0, 1.0, 1.0, 1.0));
        assert_approx_eq(ratio, 3.976_653);
    }

    #[test]
    fn adjustment_leaves_a_passing_foreground_unchanged() {
        let foreground = Srgb::new(0.0, 0.0, 0.0, 1.0);
        let background = Srgb::new(1.0, 1.0, 1.0, 1.0);

        assert_eq!(
            adjust_foreground(foreground, &[background], 4.5),
            Some(foreground)
        );
    }

    #[test]
    fn adjustment_darkens_foreground_on_a_light_background() {
        let foreground = Srgb::new(0.8, 0.5, 0.4, 1.0);
        let background = Srgb::new(1.0, 1.0, 1.0, 1.0);
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
        let foreground = Srgb::new(0.2, 0.3, 0.7, 1.0);
        let background = Srgb::new(0.0, 0.0, 0.0, 1.0);
        let adjusted = adjust_foreground(foreground, &[background], 4.5).unwrap();
        let [original_lightness, ..] = Oklch::from(foreground).components();
        let [adjusted_lightness, ..] = Oklch::from(adjusted).components();

        assert!(adjusted_lightness > original_lightness);
        assert!(contrast_ratio(adjusted, background) >= 4.5);
    }

    #[test]
    fn adjustment_returns_none_when_no_direction_can_pass_every_background() {
        let foreground = Srgb::new(0.5, 0.5, 0.5, 1.0);
        let backgrounds = [Srgb::new(0.0, 0.0, 0.0, 1.0), Srgb::new(1.0, 1.0, 1.0, 1.0)];

        assert_eq!(adjust_foreground(foreground, &backgrounds, 7.0), None);
    }
}
