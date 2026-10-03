use iced_anim::Animate;

use crate::{
    Color,
    color::Oklch,
    theme::{CategoricalSeeds, Seed},
};

use super::{
    ResolveError, Semantic, SemanticRole, TokenRole, semantic::Resolver as SemanticResolver,
};

/// Hue candidates evaluated when generating a slot.
const HUE_SAMPLES: u16 = 360;

/// Keeps generated slots hued when the band source colors are near-neutral.
const MINIMUM_CHROMA: f32 = 0.06;

/// Caps the Oklch lightness offset of generated slots from the band mean.
const MAXIMUM_LIGHTNESS_SPREAD: f32 = 0.08;

/// Below this spread, generated slots share the band's mean lightness.
const MINIMUM_LIGHTNESS_SPREAD: f32 = 0.005;

/// Discounts lightness differences so a new hue is preferred over a lighter or
/// darker copy of an existing one.
const LIGHTNESS_WEIGHT: f32 = 0.5;

/// Machado et al. (2009) protanopia simulation at full severity, in linear sRGB.
const PROTANOPIA: [[f32; 3]; 3] = [
    [0.152_286, 1.052_583, -0.204_868],
    [0.114_503, 0.786_281, 0.099_216],
    [-0.003_882, -0.048_116, 1.051_998],
];

/// Machado et al. (2009) deuteranopia simulation at full severity, in linear sRGB.
const DEUTERANOPIA: [[f32; 3]; 3] = [
    [0.367_322, 0.860_646, -0.227_968],
    [0.280_085, 0.672_501, 0.047_413],
    [-0.011_820, 0.042_940, 0.968_881],
];

/// Machado et al. (2009) tritanopia simulation at full severity, in linear sRGB.
const TRITANOPIA: [[f32; 3]; 3] = [
    [1.255_528, -0.076_749, -0.178_779],
    [-0.078_411, 0.930_809, 0.147_602],
    [0.004_733, 0.691_367, 0.303_900],
];

/// Scales simulated color-deficient distances, which shrink as hues collapse,
/// so they only dominate when a pair nearly merges for those viewers.
const DEFICIENT_VISION_SCALE: f32 = 2.0;

/// Colors for telling apart unordered categories, such as tags, chart series,
/// HTTP methods, or user avatars.
///
/// Use categorical colors when the color only needs to be *different*, not to
/// carry meaning. For states like errors or success, use the matching
/// [`Intent`](super::Intent) instead, since categorical slots may resemble
/// status colors in some themes.
///
/// Each slot is a full [`Semantic`] group with the same contrast guarantees as
/// the intents, so it can style text, badges, chips, and chart marks directly.
///
/// - Use [`get`](Self::get) or [`all`](Self::all) for ordered data such as
///   chart series; earlier slots are the most distinct.
/// - Use [`for_key`](Self::for_key) to give a named item the same color every
///   time, without maintaining a mapping.
/// - Use [`authored`](Self::authored) to limit output to colors the theme
///   author chose (i.e. weren't generated), such as for palette-faithful charts.
///
/// Themes may author up to [`Self::SLOTS`] colors through
/// [`CategoricalSeeds`]. Remaining slots are generated to match the authored
/// colors' lightness and chroma while staying as far as possible from them and
/// from the status seeds.
///
/// # Accessibility
///
/// Never use a categorical color as the only way to tell categories apart
/// ([WCAG 1.4.1 Use of Color]). Pair it with a text label, icon, or pattern.
///
/// Generated slots are chosen to stay distinct under simulated protanopia,
/// deuteranopia, and tritanopia as well as typical color vision. Because slots
/// share a similar lightness, only the first few remain reliably
/// distinguishable for people with color vision deficiency. Authored colors are used as-is, so
/// their accessibility depends on the theme's palette.
///
/// [WCAG 1.4.1 Use of Color]: https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Categorical {
    slots: [Semantic; Categorical::SLOTS],
    authored: usize,
}

impl Categorical {
    /// The number of resolved categorical slots.
    pub const SLOTS: usize = 8;

    pub(super) fn resolve(
        seed: &Seed,
        authored: &CategoricalSeeds,
        resolver: &SemanticResolver<'_>,
    ) -> Result<Self, ResolveError> {
        let bases = base_colors(seed, authored.as_slice());
        let token = |slot: usize| move |role: SemanticRole| TokenRole::Categorical(slot, role);
        // Array initialiation avoids a heap allocation since we always have 8 colors
        let mut slots = [resolver.resolve_as(bases[0], token(0))?; Self::SLOTS];
        for (slot, base) in bases.into_iter().enumerate().skip(1) {
            slots[slot] = resolver.resolve_as(base, token(slot))?;
        }

        Ok(Self {
            slots,
            authored: authored.as_slice().len(),
        })
    }

    /// Returns the colors for `index`, or `None` past [`Self::SLOTS`].
    pub fn get(&self, index: usize) -> Option<&Semantic> {
        self.slots.get(index)
    }

    /// Returns every slot, authored slots first.
    pub fn all(&self) -> &[Semantic; Self::SLOTS] {
        &self.slots
    }

    /// Returns only the slots resolved from theme-authored colors.
    pub fn authored(&self) -> &[Semantic] {
        &self.slots[..self.authored]
    }

    /// Returns a slot chosen by hashing the bytes of `key`, identical across
    /// runs, themes, platforms, and Rust releases.
    ///
    /// This is useful for getting the same categorical color for a specific key
    /// where the actual color doesn't matter much.
    ///
    /// ```
    /// # use moonveil::Theme;
    /// let categorical = Theme::default_dark().colors().categorical;
    ///
    /// let tag = categorical.for_key("design");
    /// let user = categorical.for_key(String::from("someone"));
    /// let method = categorical.for_key(b"POST");
    /// // Encode numbers with a fixed byte order so they match on every platform.
    /// let issue = categorical.for_key(42_u64.to_le_bytes());
    ///
    /// assert_eq!(tag, categorical.for_key("design"));
    /// ```
    pub fn for_key(&self, key: impl AsRef<[u8]>) -> &Semantic {
        // FNV-1a, helps guarantee stable hashing across runs and platforms.
        // Rust's default hasher may change between runs/platforms, so we don't use it here.
        let hash = key
            .as_ref()
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
            });
        // FNV mixes short keys poorly, so spread every bit into the high bits
        // with Fibonacci hashing before indexing from them.
        let hash = hash.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        &self.slots[((u128::from(hash) * Self::SLOTS as u128) >> 64) as usize]
    }
}

impl Animate for Categorical {
    fn components() -> usize {
        <[Semantic; Self::SLOTS] as Animate>::components()
    }

    fn update(&mut self, components: &mut impl Iterator<Item = f32>) {
        self.slots.update(components);
    }

    fn distance_to(&self, end: &Self) -> Vec<f32> {
        self.slots.distance_to(&end.slots)
    }

    fn lerp(&mut self, start: &Self, end: &Self, progress: f32) {
        self.slots.lerp(&start.slots, &end.slots, progress);
        // Switches with the theme definition, which also flips at the midpoint.
        self.authored = if progress < 0.5 {
            start.authored
        } else {
            end.authored
        };
    }
}

/// Returns the authored colors followed by generated colors.
///
/// Each generated color is the candidate farthest from the status seeds, the
/// authored colors, and previously generated colors, measured as the smallest
/// distance across typical, protanopic, deuteranopic, and tritanopic vision. Candidates
/// share the mean chroma of the authored colors, or of the status seeds when
/// nothing is authored, and sit at the mean lightness or one standard deviation
/// of lightness away from it. Tightly grouped palettes such as pastels
/// therefore keep a uniform lightness.
fn base_colors(seed: &Seed, authored: &[Color]) -> [Color; Categorical::SLOTS] {
    let status = [seed.accent, seed.success, seed.warning, seed.danger];
    let band = Band::of(if authored.is_empty() {
        &status
    } else {
        authored
    });
    let levels: &[f32] = if band.spread < MINIMUM_LIGHTNESS_SPREAD {
        &[0.0]
    } else {
        &[0.0, -1.0, 1.0]
    };
    let candidates: Vec<Point> = levels
        .iter()
        .flat_map(|level| {
            let lightness = (band.lightness + level * band.spread).clamp(0.0, 1.0);
            (0..HUE_SAMPLES).map(move |step| {
                let hue = f32::from(step) * 360.0 / f32::from(HUE_SAMPLES);
                Point::new(Color::from(Oklch::new(lightness, band.chroma, hue, 1.0)))
            })
        })
        .collect();

    let mut placed: Vec<Point> = status
        .iter()
        .chain(authored)
        .copied()
        .map(Point::new)
        .collect();
    let mut colors = [Color::from_rgb(0.0, 0.0, 0.0); Categorical::SLOTS];
    colors[..authored.len()].copy_from_slice(authored);

    for color in &mut colors[authored.len()..] {
        let (next, _) = candidates
            .iter()
            .map(|candidate| {
                let nearest = placed
                    .iter()
                    .map(|other| candidate.distance(other))
                    .fold(f32::INFINITY, f32::min);
                (candidate, nearest)
            })
            .reduce(|best, candidate| {
                if candidate.1 > best.1 {
                    candidate
                } else {
                    best
                }
            })
            .expect("hue candidates are never empty");
        placed.push(*next);
        *color = next.color;
    }

    colors
}

/// The lightness and chroma range shared by generated slots.
struct Band {
    lightness: f32,
    spread: f32,
    chroma: f32,
}

impl Band {
    /// Returns the mean Oklch lightness and chroma of `colors`, and the
    /// standard deviation of their lightness.
    fn of(colors: &[Color]) -> Self {
        let count = colors.len() as f32;
        let components: Vec<[f32; 4]> = colors
            .iter()
            .map(|color| Oklch::from(*color).components())
            .collect();
        let lightness = components.iter().map(|[l, ..]| l).sum::<f32>() / count;
        let chroma = components.iter().map(|[_, c, ..]| c).sum::<f32>() / count;
        let variance = components
            .iter()
            .map(|[l, ..]| (l - lightness).powi(2))
            .sum::<f32>()
            / count;

        Self {
            lightness,
            spread: variance.sqrt().min(MAXIMUM_LIGHTNESS_SPREAD),
            chroma: chroma.max(MINIMUM_CHROMA),
        }
    }
}

/// A color with Oklab coordinates as seen with typical, protanopic,
/// deuteranopic, and tritanopic vision, lightness scaled by [`LIGHTNESS_WEIGHT`].
#[derive(Clone, Copy)]
struct Point {
    color: Color,
    views: [[f32; 3]; 4],
}

impl Point {
    fn new(color: Color) -> Self {
        Self {
            color,
            views: [
                color,
                color.transform_linear(PROTANOPIA),
                color.transform_linear(DEUTERANOPIA),
                color.transform_linear(TRITANOPIA),
            ]
            .map(weighted_oklab),
        }
    }

    /// Returns the smallest distance to `other` across every simulated vision,
    /// with color-deficient distances scaled by [`DEFICIENT_VISION_SCALE`].
    fn distance(&self, other: &Self) -> f32 {
        self.views
            .iter()
            .zip(&other.views)
            .zip([
                1.0,
                DEFICIENT_VISION_SCALE,
                DEFICIENT_VISION_SCALE,
                DEFICIENT_VISION_SCALE,
            ])
            .map(|(([l, a, b], [other_l, other_a, other_b]), scale)| {
                scale * (l - other_l).hypot(a - other_a).hypot(b - other_b)
            })
            .fold(f32::INFINITY, f32::min)
    }
}

fn weighted_oklab(color: Color) -> [f32; 3] {
    let [lightness, chroma, hue, _] = Oklch::from(color).components();
    let hue = hue.to_radians();

    [
        lightness * LIGHTNESS_WEIGHT,
        chroma * hue.cos(),
        chroma * hue.sin(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Definition, Polarity, Theme};

    fn oklab_distance(first: Color, second: Color) -> f32 {
        let lab = |color: Color| {
            let [l, c, h, _] = Oklch::from(color).components();
            [l, c * h.to_radians().cos(), c * h.to_radians().sin()]
        };
        let ([l1, a1, b1], [l2, a2, b2]) = (lab(first), lab(second));
        (l1 - l2).hypot(a1 - a2).hypot(b1 - b2)
    }

    fn lightness(color: Color) -> f32 {
        Oklch::from(color).components()[0]
    }

    #[test]
    fn generated_slots_stay_distinct_for_color_vision_deficiency() {
        for polarity in [Polarity::Dark, Polarity::Light] {
            let colors = base_colors(&Definition::default_for(polarity).seed, &[]);

            for simulation in [PROTANOPIA, DEUTERANOPIA, TRITANOPIA] {
                let simulated = colors.map(|color| color.transform_linear(simulation));
                assert!(minimum_pairwise_distance(&simulated) > 0.02);
            }
        }
    }

    #[test]
    fn deficient_vision_simulations_preserve_neutrals_and_merge_red_with_green() {
        let white = Color::from_rgb(1.0, 1.0, 1.0);
        let red = Color::from(Oklch::new(0.6, 0.12, 30.0, 1.0));
        let green = Color::from(Oklch::new(0.6, 0.12, 140.0, 1.0));

        for simulation in [PROTANOPIA, DEUTERANOPIA] {
            assert!(oklab_distance(white.transform_linear(simulation), white) < 0.01);
            assert!(
                oklab_distance(
                    red.transform_linear(simulation),
                    green.transform_linear(simulation),
                ) < oklab_distance(red, green) / 2.0
            );
        }
    }

    #[test]
    fn tritanopia_simulation_preserves_neutrals_and_merges_blue_with_green() {
        let white = Color::from_rgb(1.0, 1.0, 1.0);
        let blue = Color::from(Oklch::new(0.6, 0.12, 250.0, 1.0));
        let green = Color::from(Oklch::new(0.6, 0.12, 170.0, 1.0));

        assert!(oklab_distance(white.transform_linear(TRITANOPIA), white) < 0.01);
        assert!(
            oklab_distance(
                blue.transform_linear(TRITANOPIA),
                green.transform_linear(TRITANOPIA),
            ) < oklab_distance(blue, green) / 2.0
        );
    }

    fn minimum_pairwise_distance(colors: &[Color]) -> f32 {
        let mut minimum = f32::INFINITY;
        for (index, first) in colors.iter().enumerate() {
            for second in &colors[index + 1..] {
                minimum = minimum.min(oklab_distance(*first, *second));
            }
        }
        minimum
    }

    #[test]
    fn generates_every_slot_when_nothing_is_authored() {
        for polarity in [Polarity::Dark, Polarity::Light] {
            let definition = Definition::default_for(polarity);
            let colors = base_colors(&definition.seed, &[]);
            let seed = definition.seed;
            let mut all = vec![seed.accent, seed.success, seed.warning, seed.danger];
            all.extend(colors);

            // Spacing slots for color-deficient vision costs some typical-vision spacing.
            assert!(minimum_pairwise_distance(&colors) > 0.04);
            assert!(minimum_pairwise_distance(&all) > 0.03);
        }
    }

    #[test]
    fn uniform_lightness_palettes_generate_at_the_same_lightness() {
        let definition = Definition::default_for(Polarity::Dark);
        let authored = [0.0, 120.0, 240.0].map(|hue| Color::from(Oklch::new(0.8, 0.08, hue, 1.0)));
        let colors = base_colors(&definition.seed, &authored);

        for generated in &colors[authored.len()..] {
            assert!((lightness(*generated) - 0.8).abs() < 0.01);
        }
    }

    #[test]
    fn varied_lightness_palettes_generate_varied_lightness_within_the_cap() {
        let definition = Definition::default_for(Polarity::Dark);
        let authored = [(0.5, 0.0), (0.9, 120.0), (0.5, 240.0), (0.9, 300.0)]
            .map(|(lightness, hue)| Color::from(Oklch::new(lightness, 0.08, hue, 1.0)));
        let mean = authored.iter().copied().map(lightness).sum::<f32>() / 4.0;
        let generated: Vec<f32> = base_colors(&definition.seed, &authored)[authored.len()..]
            .iter()
            .copied()
            .map(lightness)
            .collect();

        assert!(generated.iter().any(|l| (l - mean).abs() > 0.05));
        for l in generated {
            assert!((l - mean).abs() <= MAXIMUM_LIGHTNESS_SPREAD + 0.01);
        }
    }

    #[test]
    fn authored_colors_lead_and_generated_colors_fill_the_rest() {
        let definition = Definition::default_for(Polarity::Dark);
        let authored = [
            Color::from_rgb8(0xc4, 0xa7, 0xe7),
            Color::from_rgb8(0x9c, 0xcf, 0xd8),
            Color::from_rgb8(0xf6, 0xc1, 0x77),
        ];
        let colors = base_colors(&definition.seed, &authored);

        assert_eq!(colors[..3], authored);
        for generated in &colors[3..] {
            assert!(!authored.contains(generated));
        }
        assert_eq!(colors, base_colors(&definition.seed, &authored));
    }

    #[test]
    fn exposes_only_authored_slots_through_authored() {
        let authored = [
            Color::from_rgb8(0xc4, 0xa7, 0xe7),
            Color::from_rgb8(0x9c, 0xcf, 0xd8),
        ];
        let theme =
            Theme::new(Definition::default_for(Polarity::Dark).with_categorical(authored)).unwrap();
        let categorical = theme.colors().categorical;

        assert_eq!(categorical.authored().len(), 2);
        assert_eq!(categorical.authored(), &categorical.all()[..2]);
        assert!(
            Theme::default_dark()
                .colors()
                .categorical
                .authored()
                .is_empty()
        );
    }

    #[test]
    fn get_is_bounded_by_slot_count() {
        let categorical = Theme::default_dark().colors().categorical;

        assert!(categorical.get(Categorical::SLOTS - 1).is_some());
        assert!(categorical.get(Categorical::SLOTS).is_none());
    }

    #[test]
    fn for_key_is_deterministic_and_spreads_keys() {
        let dark = Theme::default_dark().colors().categorical;
        let light = Theme::default_light().colors().categorical;
        let index = |categorical: &Categorical, key: &str| {
            let slot = categorical.for_key(key);
            categorical
                .all()
                .iter()
                .position(|candidate| std::ptr::eq(candidate, slot))
                .unwrap()
        };
        let methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

        for method in methods {
            assert_eq!(index(&dark, method), index(&dark, method));
            assert_eq!(index(&dark, method), index(&light, method));
        }
        let distinct: std::collections::HashSet<_> =
            methods.iter().map(|method| index(&dark, method)).collect();
        assert!(distinct.len() > 3);

        // Single bytes are the hardest case for FNV-1a to spread.
        let letters: Vec<String> = ('a'..='z').map(String::from).collect();
        let distinct: std::collections::HashSet<_> =
            letters.iter().map(|letter| index(&dark, letter)).collect();
        assert_eq!(distinct.len(), Categorical::SLOTS);
    }

    #[test]
    fn rejects_non_opaque_authored_colors() {
        let definition = Definition::default_for(Polarity::Dark).with_categorical([
            Color::from_rgb(0.5, 0.5, 0.5),
            Color::from_rgba(0.5, 0.5, 0.5, 0.5),
        ]);

        assert!(matches!(
            Theme::new(definition),
            Err(ResolveError::NonOpaqueSeed {
                seed: "categorical[1]",
                ..
            })
        ));
    }

    #[test]
    fn categorical_seeds_from_slice_enforces_the_slot_limit() {
        let color = Color::from_rgb(0.5, 0.5, 0.5);

        assert_eq!(
            CategoricalSeeds::from_slice(&[color; 3])
                .unwrap()
                .as_slice(),
            &[color; 3]
        );
        assert!(CategoricalSeeds::from_slice(&[color; Categorical::SLOTS + 1]).is_none());
        assert_eq!(
            CategoricalSeeds::from_slice(&[]).unwrap(),
            CategoricalSeeds::EMPTY
        );
    }

    #[test]
    fn authored_count_switches_at_the_animation_midpoint() {
        let start = Theme::default_dark().colors().categorical;
        let end = Theme::new(
            Definition::default_for(Polarity::Dark)
                .with_categorical([Color::from_rgb8(0xc4, 0xa7, 0xe7)]),
        )
        .unwrap()
        .colors()
        .categorical;
        let mut categorical = start;

        categorical.lerp(&start, &end, 0.4);
        assert!(categorical.authored().is_empty());
        categorical.lerp(&start, &end, 0.6);
        assert_eq!(categorical.authored().len(), 1);
    }
}
