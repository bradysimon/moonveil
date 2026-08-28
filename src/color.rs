//! Various types of colors used in Moonveil.
//!
//! Conversions pulled from various sources:
//! - https://bottosson.github.io/posts/oklab/
//! - https://drafts.csswg.org/css-color-4/

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Srgb {
    /// The red component of the color, [0.0, 1.0].
    red: f32,
    /// The green component of the color, [0.0, 1.0].
    green: f32,
    /// The blue component of the color, [0.0, 1.0].
    blue: f32,
    /// The alpha component of the color, [0.0, 1.0].
    alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Oklab {
    /// The lightness component of the color, [0.0, 1.0].
    lightness: f32,
    /// The green-to-red component. Negative values are greener, positive values are redder.
    /// [-1.0, 1.0].
    a: f32,
    /// The blue-to-yellow component. Negative values are bluer, positive values are yellower.
    /// [-1.0, 1.0].
    b: f32,
    /// The alpha component of the color, [0.0, 1.0].
    alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Oklch {
    /// The lightness component of the color, [0.0, 1.0].
    lightness: f32,
    /// The chroma component of the color, [0.0, 1.0].
    chroma: f32,
    /// The hue component of the color, [0.0, 360.0].
    hue: f32,
    /// The alpha component of the color, [0.0, 1.0].
    alpha: f32,
}

impl From<Srgb> for Oklab {
    fn from(color: Srgb) -> Self {
        let red = srgb_to_linear(color.red);
        let green = srgb_to_linear(color.green);
        let blue = srgb_to_linear(color.blue);

        let lightness = 0.412_221_46 * red + 0.536_332_55 * green + 0.051_445_995 * blue;
        let medium = 0.211_903_5 * red + 0.680_699_5 * green + 0.107_396_96 * blue;
        let short = 0.088_302_46 * red + 0.281_718_85 * green + 0.629_978_7 * blue;

        let lightness = lightness.cbrt();
        let medium = medium.cbrt();
        let short = short.cbrt();

        Self {
            lightness: 0.210_454_26 * lightness + 0.793_617_8 * medium - 0.004_072_047 * short,
            a: 1.977_998_5 * lightness - 2.428_592_2 * medium + 0.450_593_7 * short,
            b: 0.025_904_037 * lightness + 0.782_771_77 * medium - 0.808_675_77 * short,
            alpha: color.alpha,
        }
    }
}

impl From<Oklab> for Srgb {
    fn from(color: Oklab) -> Self {
        let lightness = color.lightness + 0.396_337_78 * color.a + 0.215_803_76 * color.b;
        let medium = color.lightness - 0.105_561_346 * color.a - 0.063_854_17 * color.b;
        let short = color.lightness - 0.089_484_18 * color.a - 1.291_485_5 * color.b;

        let lightness = lightness.powi(3);
        let medium = medium.powi(3);
        let short = short.powi(3);

        Self {
            red: linear_to_srgb(
                4.076_741_7 * lightness - 3.307_711_6 * medium + 0.230_969_94 * short,
            ),
            green: linear_to_srgb(
                -1.268_438 * lightness + 2.609_757_4 * medium - 0.341_319_38 * short,
            ),
            blue: linear_to_srgb(
                -0.004_196_086_3 * lightness - 0.703_418_6 * medium + 1.707_614_7 * short,
            ),
            alpha: color.alpha,
        }
    }
}

impl From<Oklab> for Oklch {
    fn from(color: Oklab) -> Self {
        let chroma = color.a.hypot(color.b);
        let hue = if chroma <= f32::EPSILON {
            0.0
        } else {
            color.b.atan2(color.a).to_degrees().rem_euclid(360.0)
        };

        Self {
            lightness: color.lightness,
            chroma,
            hue,
            alpha: color.alpha,
        }
    }
}

impl From<Oklch> for Oklab {
    fn from(color: Oklch) -> Self {
        let hue = color.hue.to_radians();

        Self {
            lightness: color.lightness,
            a: color.chroma * hue.cos(),
            b: color.chroma * hue.sin(),
            alpha: color.alpha,
        }
    }
}

impl From<Srgb> for Oklch {
    fn from(color: Srgb) -> Self {
        Oklab::from(color).into()
    }
}

impl From<Oklch> for Srgb {
    fn from(color: Oklch) -> Self {
        Oklab::from(color).into()
    }
}

fn srgb_to_linear(channel: f32) -> f32 {
    if channel <= 0.040_45 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(channel: f32) -> f32 {
    if channel <= 0.003_130_8 {
        channel * 12.92
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
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

    fn srgba(red: u8, green: u8, blue: u8, alpha: f32) -> Srgb {
        Srgb {
            red: f32::from(red) / 255.0,
            green: f32::from(green) / 255.0,
            blue: f32::from(blue) / 255.0,
            alpha,
        }
    }

    fn assert_srgba_approx_eq(actual: Srgb, expected: Srgb) {
        assert_approx_eq(actual.red, expected.red);
        assert_approx_eq(actual.green, expected.green);
        assert_approx_eq(actual.blue, expected.blue);
        assert_approx_eq(actual.alpha, expected.alpha);
    }

    #[test]
    fn converts_srgba_red_to_known_oklab_value() {
        let color = Oklab::from(srgba(255, 0, 0, 0.75));

        assert_approx_eq(color.lightness, 0.627_955_4);
        assert_approx_eq(color.a, 0.224_863_07);
        assert_approx_eq(color.b, 0.125_846_3);
        assert_approx_eq(color.alpha, 0.75);
    }

    #[test]
    fn converts_oklab_red_to_known_oklch_value() {
        let color = Oklch::from(Oklab::from(srgba(255, 0, 0, 0.75)));

        assert_approx_eq(color.lightness, 0.627_955_4);
        assert_approx_eq(color.chroma, 0.257_683_3);
        assert_approx_eq(color.hue, 29.233_89);
        assert_approx_eq(color.alpha, 0.75);
    }

    #[test]
    fn normalizes_negative_hue() {
        let color = Oklch::from(Oklab {
            lightness: 0.5,
            a: 0.0,
            b: -0.2,
            alpha: 1.0,
        });

        assert_approx_eq(color.hue, 270.0);
    }

    #[test]
    fn uses_zero_hue_for_achromatic_colors() {
        let color = Oklch::from(Oklab {
            lightness: 0.5,
            a: 0.0,
            b: 0.0,
            alpha: 1.0,
        });

        assert_eq!(color.chroma, 0.0);
        assert_eq!(color.hue, 0.0);
    }

    #[test]
    fn srgb_oklab_roundtrip() {
        let colors = [
            srgba(25, 27, 32, 1.0),
            srgba(230, 225, 213, 0.8),
            srgba(138, 182, 200, 0.6),
            srgba(92, 194, 187, 0.4),
        ];

        for expected in colors {
            let actual = Srgb::from(Oklab::from(expected));
            assert_srgba_approx_eq(actual, expected);
        }
    }

    #[test]
    fn srgb_oklch_roundtrip() {
        let colors = [
            srgba(25, 27, 32, 1.0),
            srgba(230, 225, 213, 0.8),
            srgba(138, 182, 200, 0.6),
            srgba(92, 194, 187, 0.4),
        ];

        for expected in colors {
            let actual = Srgb::from(Oklch::from(expected));
            assert_srgba_approx_eq(actual, expected);
        }
    }
}
