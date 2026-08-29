//! Non-color appearance values and resolved effects.

use crate::{Color, Polarity, theme::Definition};
use iced::Vector;
use iced_anim::Animate;

/// Non-layout presentation values that may be consumed by widget styles.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// Border widths used by decoration, controls, and focus indicators.
///
/// Values are measured in logical pixels.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// A resolved shadow that maps directly to Iced's shadow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    pub color: Color,
    pub offset: Vector<f32>,
    pub blur_radius: f32,
}

impl From<Shadow> for iced::Shadow {
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadows {
    /// Anchored menus, pick lists, popovers, and floating tooltips.
    pub popover: Shadow,
    /// Dialog surfaces shown above a scrim.
    pub modal: Shadow,
    /// A compact directional cue for content currently being dragged.
    pub drag_preview: Shadow,
}

impl Appearance {
    pub(super) fn resolve(definition: &Definition) -> Self {
        let shadow_color = definition.seed.shade.with_max_chroma(0.02);
        let (popover_alpha, modal_alpha, drag_preview_alpha) = match definition.polarity {
            Polarity::Dark => (0.50, 0.62, 0.46),
            Polarity::Light => (0.20, 0.26, 0.18),
        };

        Self {
            radius: Radii {
                xs: 3.0,
                sm: 5.0,
                md: 8.0,
                lg: 12.0,
                full: 999.0,
            },
            border: BorderWidths {
                hairline: 1.0,
                control: 1.0,
                focus: 2.0,
            },
            shadow: Shadows {
                popover: shadow(shadow_color, popover_alpha, 4.0, 18.0),
                modal: shadow(shadow_color, modal_alpha, 8.0, 24.0),
                drag_preview: shadow(shadow_color, drag_preview_alpha, 6.0, 16.0),
            },
        }
    }
}

impl Animate for Appearance {
    fn components() -> usize {
        <Color as Animate>::components() * 3
    }

    fn update(&mut self, components: &mut impl Iterator<Item = f32>) {
        self.shadow.popover.color.update(components);
        self.shadow.modal.color.update(components);
        self.shadow.drag_preview.color.update(components);
    }

    fn distance_to(&self, end: &Self) -> Vec<f32> {
        let mut distance = self
            .shadow
            .popover
            .color
            .distance_to(&end.shadow.popover.color);
        distance.extend(self.shadow.modal.color.distance_to(&end.shadow.modal.color));
        distance.extend(
            self.shadow
                .drag_preview
                .color
                .distance_to(&end.shadow.drag_preview.color),
        );
        distance
    }

    fn lerp(&mut self, start: &Self, end: &Self, progress: f32) {
        *self = if progress < 0.5 { *start } else { *end };
        self.shadow.popover.color.lerp(
            &start.shadow.popover.color,
            &end.shadow.popover.color,
            progress,
        );
        self.shadow
            .modal
            .color
            .lerp(&start.shadow.modal.color, &end.shadow.modal.color, progress);
        self.shadow.drag_preview.color.lerp(
            &start.shadow.drag_preview.color,
            &end.shadow.drag_preview.color,
            progress,
        );
    }
}

fn shadow(color: Color, alpha: f32, offset_y: f32, blur_radius: f32) -> Shadow {
    let [red, green, blue, _] = color.components();

    Shadow {
        color: Color::from_rgba(red, green, blue, alpha),
        offset: Vector::new(0.0, offset_y),
        blur_radius,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Oklch;

    #[test]
    fn resolves_default_geometry() {
        let dark = Appearance::resolve(&Definition::default_for(Polarity::Dark));

        assert_eq!(dark.radius.md, 8.0);
        assert_eq!(dark.border.focus, 2.0);
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
}
