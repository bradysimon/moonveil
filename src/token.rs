//! Resolved color tokens.

use crate::Color;

/// Neutral planes used to establish visual depth and grouping.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surfaces {
    /// Recessed content such as code wells and terminal output.
    pub sunken: Color,
    /// Receding application chrome such as sidebars.
    pub canvas: Color,
    /// Local recesses such as tracks and segmented-control backgrounds.
    pub inset: Color,
    /// The primary content plane.
    pub surface: Color,
    /// Inline panels and grouped content; does not imply a shadow.
    pub raised: Color,
    /// Menus, popovers, and other content placed above another plane.
    pub overlay: Color,
    /// Editable content and dense data planes.
    pub field: Color,
    /// A translucent backdrop placed between modal and underlying content.
    pub scrim: Color,
}

/// Foregrounds grouped by content emphasis.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Content {
    /// Headings, values, and primary document content.
    pub primary: Color,
    /// Body copy, labels, active controls, and icons.
    pub secondary: Color,
    /// Readable metadata, hints, and placeholders.
    pub muted: Color,
    /// Nonessential icons and large incidental text.
    pub decorative: Color,
    /// Unavailable controls and values only.
    pub disabled: Color,
    /// Content shown on strongly inverted neutral surfaces.
    pub inverse: Color,
}

/// Boundaries grouped by visual and interaction intent.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Borders {
    /// Decorative separation where shape is already apparent.
    pub subtle: Color,
    /// Ordinary boundaries for grouped regions and panels.
    pub standard: Color,
    /// Clear edges for fields and controls.
    pub strong: Color,
    /// Keyboard focus indication where focus state is available.
    pub focus: Color,
    /// Selected and checked outlines.
    pub selected: Color,
}

/// Temporary overlays for interaction states.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interaction {
    /// Overlay applied while a pointer hovers over an interactive region.
    pub hover: Color,
    /// Overlay applied while an interactive region is pressed.
    pub pressed: Color,
    /// Persistent overlay for selected content.
    pub selected: Color,
    /// Hover overlay for selected content.
    pub selected_hover: Color,
    /// Overlay for content currently being dragged.
    pub dragged: Color,
    /// Overlay identifying a valid drop destination.
    pub drop_target: Color,
}

/// A fill and the text or icon color guaranteed against it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pair {
    /// The background or fill color.
    pub color: Color,
    /// Text and icon color guaranteed against [`Self::color`].
    pub text: Color,
}

/// Resolved fill pairs for each interactive state.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fill {
    /// The default fill and foreground pair.
    pub active: Pair,
    /// The fill and foreground pair while hovered.
    pub hovered: Pair,
    /// The fill and foreground pair while pressed.
    pub pressed: Pair,
}

/// Resolved roles for one semantic intent.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Semantic {
    /// Semantic text and icons shown directly on neutral surfaces.
    pub foreground: Color,
    /// Essential non-text marks such as range fills and status graphics.
    pub indicator: Color,
    /// High-emphasis actions and compact status fills.
    pub solid: Fill,
    /// Alerts, badges, selected rows, and low-emphasis semantic actions.
    pub soft: Fill,
    /// A visible edge for semantic regions placed on neutral surfaces.
    pub border: Color,
}

/// All resolved color tokens for a theme.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colors {
    /// Neutral planes used throughout the interface.
    pub surfaces: Surfaces,
    /// Foregrounds grouped by content emphasis.
    pub content: Content,
    /// Decorative, control, focus, and selection boundaries.
    pub borders: Borders,
    /// Temporary and persistent interaction overlays.
    pub interaction: Interaction,
    /// Accent and primary-action roles.
    pub accent: Semantic,
    /// Successful and positive-state roles.
    pub success: Semantic,
    /// Caution and warning-state roles.
    pub warning: Semantic,
    /// Destructive, invalid, and error-state roles.
    pub danger: Semantic,
    /// Informational and neutral-notice roles.
    pub info: Semantic,
}
