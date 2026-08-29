//! Various types of colors used in Moonveil.
//!
//! Conversions pulled from various sources:
//! - <https://bottosson.github.io/posts/oklab/>
//! - <https://drafts.csswg.org/css-color-4/>

/// A color in the sRGB color space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
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

impl Color {
    /// Creates an opaque color from sRGB components in range `[0.0, 1.0]`.
    pub const fn from_rgb(red: f32, green: f32, blue: f32) -> Self {
        Self::from_rgba(red, green, blue, 1.0)
    }

    /// Creates a color from sRGB components in range `[0.0, 1.0]`.
    pub const fn from_rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        assert!(red >= 0.0 && red <= 1.0, "red must be in [0.0, 1.0]");
        assert!(green >= 0.0 && green <= 1.0, "green must be in [0.0, 1.0]");
        assert!(blue >= 0.0 && blue <= 1.0, "blue must be in [0.0, 1.0]");
        assert!(alpha >= 0.0 && alpha <= 1.0, "alpha must be in [0.0, 1.0]");

        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Creates a new [`Color`].
    pub(crate) fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self::from_rgba(red, green, blue, alpha)
    }

    /// Gets the `red`, `green`, `blue`, and `alpha` components of this color.
    pub const fn components(self) -> [f32; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }

    /// Mixes two sRGB colors in the [`Oklab`] color space.
    /// Clamps `amount` to the range [0.0, 1.0].
    pub(crate) fn mix_oklab(self, other: Self, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);

        let first = Oklab::from(self);
        let second = Oklab::from(other);

        Oklab::new(
            lerp(first.lightness, second.lightness, amount),
            lerp(first.a, second.a, amount),
            lerp(first.b, second.b, amount),
            lerp(first.alpha, second.alpha, amount),
        )
        .into()
    }

    /// Reduces [`Oklch`] chroma to `maximum` while preserving lightness and hue.
    pub(crate) fn with_max_chroma(self, maximum: f32) -> Self {
        let [lightness, chroma, hue, alpha] = Oklch::from(self).components();

        if chroma <= maximum {
            self
        } else {
            Oklch::new(lightness, maximum, hue, alpha).into()
        }
    }

    /// Attempts to convert an [`Oklab`] color to sRGB.
    /// Returns `None` if the color is out of the sRGB gamut.
    fn try_from(color: Oklab) -> Option<Self> {
        let lightness = color.lightness + 0.396_337_78 * color.a + 0.215_803_76 * color.b;
        let medium = color.lightness - 0.105_561_346 * color.a - 0.063_854_17 * color.b;
        let short = color.lightness - 0.089_484_18 * color.a - 1.291_485_5 * color.b;

        let lightness = lightness.powi(3);
        let medium = medium.powi(3);
        let short = short.powi(3);

        let red =
            linear_to_srgb(4.076_741_7 * lightness - 3.307_711_6 * medium + 0.230_969_94 * short);
        let green =
            linear_to_srgb(-1.268_438 * lightness + 2.609_757_4 * medium - 0.341_319_38 * short);
        let blue = linear_to_srgb(
            -0.004_196_086_3 * lightness - 0.703_418_6 * medium + 1.707_614_7 * short,
        );

        if [red, green, blue]
            .into_iter()
            .all(|channel| (0.0..=1.0).contains(&channel))
        {
            Some(Self::new(red, green, blue, color.alpha))
        } else {
            None
        }
    }
}

impl From<Color> for iced::Color {
    fn from(color: Color) -> Self {
        let [red, green, blue, alpha] = color.components();
        Self::from_rgba(red, green, blue, alpha)
    }
}

impl iced_anim::Animate for Color {
    fn components() -> usize {
        4
    }

    fn update(&mut self, components: &mut impl Iterator<Item = f32>) {
        let color = Oklab::from(*self);
        *self = Oklab::new(
            (color.lightness + components.next().unwrap()).clamp(0.0, 1.0),
            (color.a + components.next().unwrap()).clamp(-1.0, 1.0),
            (color.b + components.next().unwrap()).clamp(-1.0, 1.0),
            (color.alpha + components.next().unwrap()).clamp(0.0, 1.0),
        )
        .into();
    }

    fn distance_to(&self, end: &Self) -> Vec<f32> {
        let start = Oklab::from(*self);
        let end = Oklab::from(*end);

        vec![
            start.lightness - end.lightness,
            start.a - end.a,
            start.b - end.b,
            start.alpha - end.alpha,
        ]
    }

    fn lerp(&mut self, start: &Self, end: &Self, progress: f32) {
        *self = if progress <= 0.0 {
            *start
        } else if progress >= 1.0 {
            *end
        } else {
            start.mix_oklab(*end, progress)
        };
    }
}

impl Oklab {
    /// Creates a new [`Oklab`] color.
    const fn new(lightness: f32, a: f32, b: f32, alpha: f32) -> Self {
        debug_assert!(
            lightness >= 0.0 && lightness <= 1.0,
            "Lightness must be in range [0.0, 1.0]"
        );
        debug_assert!(a >= -1.0 && a <= 1.0, "A must be in range [-1.0, 1.0]");
        debug_assert!(b >= -1.0 && b <= 1.0, "B must be in range [-1.0, 1.0]");
        debug_assert!(
            alpha >= 0.0 && alpha <= 1.0,
            "Alpha must be in range [0.0, 1.0]"
        );

        Self {
            lightness,
            a,
            b,
            alpha,
        }
    }
}

impl Oklch {
    /// Creates a new [`Oklch`] color.
    pub(crate) const fn new(lightness: f32, chroma: f32, hue: f32, alpha: f32) -> Self {
        debug_assert!(
            lightness >= 0.0 && lightness <= 1.0,
            "Lightness must be in range [0.0, 1.0]"
        );
        debug_assert!(
            chroma >= 0.0 && chroma <= 1.0,
            "Chroma must be in range [0.0, 1.0]"
        );
        debug_assert!(
            hue >= 0.0 && hue <= 360.0,
            "Hue must be in range [0.0, 360.0]"
        );
        debug_assert!(
            alpha >= 0.0 && alpha <= 1.0,
            "Alpha must be in range [0.0, 1.0]"
        );

        Self {
            lightness,
            chroma,
            hue,
            alpha,
        }
    }

    /// Gets the `lightness`, `chroma`, `hue`, and `alpha` components of this color.
    pub(crate) fn components(self) -> [f32; 4] {
        [self.lightness, self.chroma, self.hue, self.alpha]
    }
}

impl From<Color> for Oklab {
    fn from(color: Color) -> Self {
        let red = srgb_to_linear(color.red);
        let green = srgb_to_linear(color.green);
        let blue = srgb_to_linear(color.blue);

        let lightness = 0.412_221_46 * red + 0.536_332_55 * green + 0.051_445_995 * blue;
        let medium = 0.211_903_5 * red + 0.680_699_5 * green + 0.107_396_96 * blue;
        let short = 0.088_302_46 * red + 0.281_718_85 * green + 0.629_978_7 * blue;

        let lightness = lightness.cbrt();
        let medium = medium.cbrt();
        let short = short.cbrt();

        Self::new(
            0.210_454_26 * lightness + 0.793_617_8 * medium - 0.004_072_047 * short,
            1.977_998_5 * lightness - 2.428_592_2 * medium + 0.450_593_7 * short,
            0.025_904_037 * lightness + 0.782_771_77 * medium - 0.808_675_77 * short,
            color.alpha,
        )
    }
}

impl From<Oklab> for Color {
    fn from(color: Oklab) -> Self {
        if let Some(candidate) = Self::try_from(color) {
            return candidate;
        }

        let color = Oklch::from(color);
        let mut low = 0.0;
        let mut high = color.chroma;

        for _ in 0..24 {
            let chroma = (low + high) / 2.0;
            let candidate = Self::try_from(Oklab::from(Oklch::new(
                color.lightness,
                chroma,
                color.hue,
                color.alpha,
            )));

            if candidate.is_some() {
                low = chroma;
            } else {
                high = chroma;
            }
        }

        Self::try_from(Oklab::from(Oklch::new(
            color.lightness,
            low,
            color.hue,
            color.alpha,
        )))
        .expect("zero chroma must be inside the sRGB gamut")
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

        Self::new(color.lightness, chroma, hue, color.alpha)
    }
}

impl From<Oklch> for Oklab {
    fn from(color: Oklch) -> Self {
        let hue = color.hue.to_radians();

        Self::new(
            color.lightness,
            color.chroma * hue.cos(),
            color.chroma * hue.sin(),
            color.alpha,
        )
    }
}

impl From<Color> for Oklch {
    fn from(color: Color) -> Self {
        Oklab::from(color).into()
    }
}

impl From<Oklch> for Color {
    fn from(color: Oklch) -> Self {
        Oklab::from(color).into()
    }
}

fn lerp(first: f32, second: f32, amount: f32) -> f32 {
    first + (second - first) * amount
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
    use iced_anim::Animate;

    const EPSILON: f32 = 0.000_01;

    fn assert_approx_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= EPSILON,
            "expected {expected}, got {actual}"
        );
    }

    fn srgba(red: u8, green: u8, blue: u8, alpha: f32) -> Color {
        Color::new(
            f32::from(red) / 255.0,
            f32::from(green) / 255.0,
            f32::from(blue) / 255.0,
            alpha,
        )
    }

    fn assert_srgba_approx_eq(actual: Color, expected: Color) {
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
        let color = Oklch::from(Oklab::new(0.5, 0.0, -0.2, 1.0));

        assert_approx_eq(color.hue, 270.0);
    }

    #[test]
    fn uses_zero_hue_for_achromatic_colors() {
        let color = Oklch::from(Oklab::new(0.5, 0.0, 0.0, 1.0));

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
            let actual = Color::from(Oklab::from(expected));
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
            let actual = Color::from(Oklch::from(expected));
            assert_srgba_approx_eq(actual, expected);
        }
    }

    #[test]
    fn mix_oklab_clamps_amount_and_preserves_endpoints() {
        let first = srgba(25, 27, 32, 0.25);
        let second = srgba(230, 225, 213, 0.75);

        assert_srgba_approx_eq(first.mix_oklab(second, -0.5), first);
        assert_srgba_approx_eq(first.mix_oklab(second, 0.0), first);
        assert_srgba_approx_eq(first.mix_oklab(second, 1.0), second);
        assert_srgba_approx_eq(first.mix_oklab(second, 1.5), second);
    }

    #[test]
    fn mix_oklab_interpolates_in_oklab_and_includes_alpha() {
        let mixed = srgba(0, 0, 0, 0.25).mix_oklab(srgba(255, 255, 255, 0.75), 0.5);

        assert_srgba_approx_eq(
            mixed,
            Color::new(0.388_572_87, 0.388_572_87, 0.388_572_87, 0.5),
        );
    }

    #[test]
    fn converts_to_iced_color() {
        let color = Color::from_rgba(0.1, 0.2, 0.3, 0.4);
        let iced = iced::Color::from(color);

        assert_eq!([iced.r, iced.g, iced.b, iced.a], color.components());
    }

    #[test]
    fn animation_uses_oklab_components() {
        let start = Color::from_rgb(0.1, 0.2, 0.3);
        let end = Color::from_rgb(0.8, 0.7, 0.6);
        let mut interpolated = start;

        interpolated.lerp(&start, &end, 0.5);

        assert_srgba_approx_eq(interpolated, start.mix_oklab(end, 0.5));
        assert_eq!(
            start.distance_to(&end).len(),
            <Color as Animate>::components()
        );
    }

    #[test]
    fn gamut_mapping_reduces_chroma_without_shifting_lightness_or_hue() {
        let source = Oklch::new(0.7, 0.4, 40.0, 0.75);
        let mapped = Color::from(source);
        let mapped_oklch = Oklch::from(mapped);

        assert!(mapped_oklch.chroma < source.chroma);
        assert_approx_eq(mapped_oklch.lightness, source.lightness);
        assert!((mapped_oklch.hue - source.hue).abs() <= 0.001);
        assert_approx_eq(mapped_oklch.alpha, source.alpha);
    }

    #[test]
    #[should_panic]
    fn color_rejects_out_of_range_components() {
        Color::new(1.1, 0.0, 0.0, 1.0);
    }
}
