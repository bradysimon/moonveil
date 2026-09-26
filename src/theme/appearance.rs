//! Non-color appearance values and resolved effects.

use crate::{Color, Polarity, theme::Definition};
use iced_anim::Animate;
use iced_core::Vector;

/// Non-layout presentation values that may be consumed by widget styles.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
pub struct Appearance {
    /// Corner radius scale.
    pub radius: Radii,
    /// Border width scale.
    pub border: BorderWidths,
    /// Resolved shadow roles.
    pub shadow: Shadows,
}

/// Corner radii used by controls and surfaces.
///
/// Values are measured in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
pub struct Radii {
    /// Tight rounding for compact controls and small details.
    pub xs: f32,
    /// Standard rounding for buttons, fields, and similar controls.
    pub sm: f32,
    /// Moderate rounding for panels and grouped content.
    pub md: f32,
    /// Broad rounding for large surfaces such as dialogs.
    pub lg: f32,
    /// Effectively complete rounding for pills and circular shapes.
    pub full: f32,
}

impl Radii {
    /// Square corners everywhere, including pills and tracks.
    pub const SHARP: Self = Self {
        xs: 0.0,
        sm: 0.0,
        md: 0.0,
        lg: 0.0,
        full: 0.0,
    };

    /// Moonveil's default rounding.
    pub const STANDARD: Self = Self {
        xs: 3.0,
        sm: 5.0,
        md: 8.0,
        lg: 12.0,
        full: 999.0,
    };

    /// Softer rounding for friendlier, more tactile interfaces.
    pub const ROUND: Self = Self {
        xs: 4.0,
        sm: 8.0,
        md: 12.0,
        lg: 16.0,
        full: 999.0,
    };

    /// Returns the radius of a container whose corners stay concentric with an
    /// `inner` radius placed `inset` pixels inside it.
    ///
    /// Square inner corners stay square so sharp themes remain sharp.
    pub fn around(inner: f32, inset: f32) -> f32 {
        if inner > 0.0 { inner + inset } else { 0.0 }
    }

    /// Returns the radius of content placed `inset` pixels inside an `outer`
    /// radius so both curves stay concentric.
    pub fn within(outer: f32, inset: f32) -> f32 {
        (outer - inset).max(0.0)
    }
}

impl Default for Radii {
    fn default() -> Self {
        Self::STANDARD
    }
}

/// Border widths used by decoration, controls, and focus indicators.
///
/// Values are measured in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
pub struct BorderWidths {
    /// Thin decorative separators and low-emphasis edges.
    ///
    /// A hairline must not be the only visual boundary identifying a control.
    pub hairline: f32,
    /// Standard visible boundaries for controls such as buttons and fields.
    pub control: f32,
    /// Keyboard focus rings where the widget exposes focus state.
    pub focus: f32,
}

impl BorderWidths {
    /// Moonveil's default border widths.
    pub const STANDARD: Self = Self {
        hairline: 1.0,
        control: 1.0,
        focus: 2.0,
    };

    /// Heavier control boundaries and focus rings for high-contrast themes.
    pub const STRONG: Self = Self {
        hairline: 1.0,
        control: 1.5,
        focus: 3.0,
    };
}

impl Default for BorderWidths {
    fn default() -> Self {
        Self::STANDARD
    }
}

/// Authored geometry and opacity of a single shadow role.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Depth {
    /// Downward offset in logical pixels.
    pub offset_y: f32,
    /// Blur radius in logical pixels.
    pub blur_radius: f32,
    /// Opacity applied to the theme's shade color.
    pub opacity: f32,
}

impl Depth {
    /// No shadow.
    pub const NONE: Self = Self::new(0.0, 0.0, 0.0);

    /// Creates a shadow depth.
    pub const fn new(offset_y: f32, blur_radius: f32, opacity: f32) -> Self {
        Self {
            offset_y,
            blur_radius,
            opacity,
        }
    }
}

/// Authored shadows for content placed above another plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Elevation {
    /// Raised containers and cards that sit slightly above their surface.
    pub raised: Depth,
    /// Anchored menus, pick lists, popovers, and floating tooltips.
    pub popover: Depth,
    /// Dialog surfaces shown above a scrim.
    pub modal: Depth,
    /// Content currently being dragged.
    pub drag_preview: Depth,
}

impl Elevation {
    /// No shadows; elevated content relies on surfaces and borders alone.
    pub const FLAT: Self = Self {
        raised: Depth::NONE,
        popover: Depth::NONE,
        modal: Depth::NONE,
        drag_preview: Depth::NONE,
    };

    /// Moonveil's default shadows, which are stronger on dark themes so they
    /// stay visible against dark surfaces.
    pub const fn standard(polarity: Polarity) -> Self {
        match polarity {
            Polarity::Dark => Self {
                raised: Depth::new(2.0, 8.0, 0.30),
                popover: Depth::new(4.0, 18.0, 0.50),
                modal: Depth::new(8.0, 24.0, 0.62),
                drag_preview: Depth::new(6.0, 16.0, 0.46),
            },
            Polarity::Light => Self {
                raised: Depth::new(2.0, 8.0, 0.10),
                popover: Depth::new(4.0, 18.0, 0.20),
                modal: Depth::new(8.0, 24.0, 0.26),
                drag_preview: Depth::new(6.0, 16.0, 0.18),
            },
        }
    }
}

/// A resolved shadow that maps directly to Iced's shadow.
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
pub struct Shadow {
    pub color: Color,
    pub offset: Vector<f32>,
    pub blur_radius: f32,
}

impl Shadow {
    fn resolve(color: Color, depth: Depth) -> Self {
        let [red, green, blue, _] = color.components();

        Self {
            color: Color::from_rgba(red, green, blue, depth.opacity),
            offset: Vector::new(0.0, depth.offset_y),
            blur_radius: depth.blur_radius,
        }
    }
}

impl From<Shadow> for iced_core::Shadow {
    fn from(shadow: Shadow) -> Self {
        Self {
            color: shadow.color.into(),
            offset: shadow.offset,
            blur_radius: shadow.blur_radius,
        }
    }
}

/// Shadows for content that is placed above another plane.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
pub struct Shadows {
    /// Raised containers and cards that sit slightly above their surface.
    pub raised: Shadow,
    /// Anchored menus, pick lists, popovers, and floating tooltips.
    pub popover: Shadow,
    /// Dialog surfaces shown above a scrim.
    pub modal: Shadow,
    /// A compact directional cue for content currently being dragged.
    pub drag_preview: Shadow,
}

impl Appearance {
    pub(super) fn resolve(definition: &Definition) -> Self {
        let color = definition.seed.shade.with_max_chroma(0.02);
        let elevation = definition.elevation;

        Self {
            radius: definition.radius,
            border: definition.border,
            shadow: Shadows {
                raised: Shadow::resolve(color, elevation.raised),
                popover: Shadow::resolve(color, elevation.popover),
                modal: Shadow::resolve(color, elevation.modal),
                drag_preview: Shadow::resolve(color, elevation.drag_preview),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Oklch;

    #[test]
    fn resolves_default_geometry() {
        let dark = Appearance::resolve(&Definition::default_for(Polarity::Dark));

        assert_eq!(dark.radius, Radii::STANDARD);
        assert_eq!(dark.border, BorderWidths::STANDARD);
    }

    /// Dark vs. light shadows have different alphas.
    #[test]
    fn uses_polarity_specific_shadow_opacity() {
        let dark = Appearance::resolve(&Definition::default_for(Polarity::Dark));
        let light = Appearance::resolve(&Definition::default_for(Polarity::Light));

        assert_eq!(dark.shadow.popover.color.components()[3], 0.50);
        assert_eq!(light.shadow.popover.color.components()[3], 0.20);
    }

    #[test]
    fn limits_shadow_chroma() {
        let dark = Appearance::resolve(&Definition::default_for(Polarity::Dark));

        assert!(Oklch::from(dark.shadow.popover.color).components()[1] <= 0.020_001);
    }

    #[test]
    fn flat_elevation_resolves_invisible_shadows() {
        let definition = Definition::default_for(Polarity::Dark).with_elevation(Elevation::FLAT);
        let shadows = Appearance::resolve(&definition).shadow;

        for shadow in [
            shadows.raised,
            shadows.popover,
            shadows.modal,
            shadows.drag_preview,
        ] {
            assert_eq!(shadow.color.components()[3], 0.0);
            assert_eq!(shadow.blur_radius, 0.0);
            assert_eq!(shadow.offset, Vector::ZERO);
        }
    }

    /// Ensures that concentric radii calculations follow the inset logic correctly.
    #[test]
    fn concentric_radii_follow_inset() {
        assert_eq!(Radii::around(5.0, 4.0), 9.0);
        assert_eq!(Radii::within(9.0, 4.0), 5.0);
        assert_eq!(Radii::within(3.0, 4.0), 0.0);
    }

    #[test]
    fn concentric_radii_keep_square_corners_square() {
        assert_eq!(Radii::around(0.0, 4.0), 0.0);
    }
}
