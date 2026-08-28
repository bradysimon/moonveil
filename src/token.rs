//! Resolved color tokens.

use std::{error::Error, fmt};

use crate::Color;

mod resolve;
mod semantic;

/// A content token identified in a resolution error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentRole {
    Primary,
    Secondary,
    Muted,
    Decorative,
    Disabled,
    Inverse,
}

/// A border token identified in a resolution error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BorderRole {
    Subtle,
    Standard,
    Strong,
    Focus,
    Selected,
}

/// The semantic meaning carried by a semantic color family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticIntent {
    Accent,
    Success,
    Warning,
    Danger,
    Info,
}

/// A token within a semantic color family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticRole {
    Foreground,
    Indicator,
    Solid,
    Soft,
    Border,
}

/// A resolved token identified in a resolution error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TokenRole {
    Content(ContentRole),
    Border(BorderRole),
    Semantic(SemanticIntent, SemanticRole),
}

impl fmt::Display for ContentRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Muted => "muted",
            Self::Decorative => "decorative",
            Self::Disabled => "disabled",
            Self::Inverse => "inverse",
        })
    }
}

impl fmt::Display for BorderRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Subtle => "subtle",
            Self::Standard => "standard",
            Self::Strong => "strong",
            Self::Focus => "focus",
            Self::Selected => "selected",
        })
    }
}

impl fmt::Display for SemanticIntent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Accent => "accent",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
            Self::Info => "info",
        })
    }
}

impl fmt::Display for SemanticRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Foreground => "foreground",
            Self::Indicator => "indicator",
            Self::Solid => "solid",
            Self::Soft => "soft",
            Self::Border => "border",
        })
    }
}

impl fmt::Display for TokenRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Content(role) => write!(formatter, "content.{role}"),
            Self::Border(role) => write!(formatter, "borders.{role}"),
            Self::Semantic(intent, role) => write!(formatter, "{intent}.{role}"),
        }
    }
}

/// An error encountered while deriving or validating resolved color tokens.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ResolveError {
    /// An authored seed that must be opaque contains transparency.
    NonOpaqueSeed {
        /// The name of the invalid seed.
        seed: &'static str,
        /// The authored alpha value.
        alpha: f32,
    },
    /// A contrast target is outside its supported range.
    InvalidTarget {
        /// The name of the invalid target.
        target: &'static str,
        /// The authored target value.
        value: f32,
    },
    /// Two adjacent neutral surfaces are not ordered by perceptual lightness.
    SurfaceOrder {
        /// The darker surface in the required ordering.
        darker: &'static str,
        /// The lighter surface in the required ordering.
        lighter: &'static str,
    },
    /// Two adjacent neutral surfaces are too close in perceptual lightness.
    SurfaceSeparation {
        /// The first adjacent surface.
        first: &'static str,
        /// The second adjacent surface.
        second: &'static str,
        /// Their resolved Oklch lightness difference.
        difference: f32,
        /// The minimum accepted Oklch lightness difference.
        minimum: f32,
    },
    /// No color could satisfy a token's contrast contract.
    UnsatisfiableContrast {
        /// The token being resolved.
        token: TokenRole,
        /// The surfaces or fills the token must contrast with.
        against: &'static str,
        /// The required WCAG contrast ratio.
        minimum_ratio: f32,
    },
    /// A resolved token failed final contrast validation.
    ContrastViolation {
        /// The token being validated.
        token: TokenRole,
        /// The specific surface or fill that failed.
        background: &'static str,
        /// The measured WCAG contrast ratio.
        actual_ratio: f32,
        /// The required WCAG contrast ratio.
        minimum_ratio: f32,
    },
}

impl fmt::Display for ResolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonOpaqueSeed { seed, alpha } => {
                write!(
                    formatter,
                    "seed `{seed}` must be opaque, but has alpha {alpha}"
                )
            }
            Self::InvalidTarget { target, value } => write!(
                formatter,
                "contrast target `{target}` must be in [1, 21], but is {value}"
            ),
            Self::SurfaceOrder { darker, lighter } => write!(
                formatter,
                "surface `{darker}` must be perceptually darker than `{lighter}`"
            ),
            Self::SurfaceSeparation {
                first,
                second,
                difference,
                minimum,
            } => write!(
                formatter,
                "surfaces `{first}` and `{second}` differ by {difference:.4} Oklch lightness; at least {minimum:.4} is required"
            ),
            Self::UnsatisfiableContrast {
                token,
                against,
                minimum_ratio,
            } => write!(
                formatter,
                "token `{token}` cannot reach {minimum_ratio}:1 contrast against {against}"
            ),
            Self::ContrastViolation {
                token,
                background,
                actual_ratio,
                minimum_ratio,
            } => write!(
                formatter,
                "token `{token}` has {actual_ratio:.3}:1 contrast against `{background}`; {minimum_ratio}:1 is required"
            ),
        }
    }
}

impl Error for ResolveError {}

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
