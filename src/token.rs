//! Resolved color tokens.

use std::{error::Error, fmt};

use crate::Color;
use iced_anim::Animate;

mod categorical;
mod resolve;
mod semantic;

pub use categorical::Categorical;

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

/// An opaque neutral plane used to establish visual depth and grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Surface {
    /// Recessed content such as code wells and terminal output.
    Sunken,
    /// Receding application chrome such as sidebars.
    Canvas,
    /// Local recesses such as tracks and segmented-control backgrounds.
    Inset,
    /// The primary content plane.
    Surface,
    /// Inline panels and grouped content without an implied shadow.
    Raised,
    /// Menus, popovers, and other content placed above another plane.
    Overlay,
    /// Editable content and dense data planes.
    Field,
}

/// A generic interaction overlay state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Interaction {
    /// A pointer is hovering over an interactive region.
    Hover,
    /// An interactive region is being pressed.
    Pressed,
    /// Content is persistently selected.
    Selected,
    /// Selected content is being hovered.
    SelectedHover,
    /// Text is selected in an editable field.
    Selection,
    /// Content is currently being dragged.
    Dragged,
    /// A region is a valid drop destination.
    DropTarget,
}

/// The meaning carried by a color family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Intent {
    Accent,
    Success,
    Warning,
    Danger,
}

impl Intent {
    pub const ALL: [Intent; 4] = [
        Intent::Accent,
        Intent::Success,
        Intent::Warning,
        Intent::Danger,
    ];
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

/// A syntax highlighting token identified in a resolution error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SyntaxRole {
    Keyword,
    TypeName,
    Function,
    String,
    Constant,
    Comment,
}

impl SyntaxRole {
    pub const ALL: [SyntaxRole; 6] = [
        SyntaxRole::Keyword,
        SyntaxRole::TypeName,
        SyntaxRole::Function,
        SyntaxRole::String,
        SyntaxRole::Constant,
        SyntaxRole::Comment,
    ];
}

/// A resolved token identified in a resolution error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TokenRole {
    Content(ContentRole),
    Border(BorderRole),
    Semantic(Intent, SemanticRole),
    /// A role within the categorical slot at the given index.
    Categorical(usize, SemanticRole),
    Syntax(SyntaxRole),
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

impl fmt::Display for Intent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Accent => "accent",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
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

impl fmt::Display for SyntaxRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Keyword => "keyword",
            Self::TypeName => "type_name",
            Self::Function => "function",
            Self::String => "string",
            Self::Constant => "constant",
            Self::Comment => "comment",
        })
    }
}

impl fmt::Display for TokenRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Content(role) => write!(formatter, "content.{role}"),
            Self::Border(role) => write!(formatter, "borders.{role}"),
            Self::Semantic(intent, role) => write!(formatter, "{intent}.{role}"),
            Self::Categorical(slot, role) => write!(formatter, "categorical.{slot}.{role}"),
            Self::Syntax(role) => write!(formatter, "syntax.{role}"),
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
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
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

impl Surfaces {
    /// Returns the resolved color for an opaque neutral surface.
    pub const fn get(&self, surface: Surface) -> Color {
        match surface {
            Surface::Sunken => self.sunken,
            Surface::Canvas => self.canvas,
            Surface::Inset => self.inset,
            Surface::Surface => self.surface,
            Surface::Raised => self.raised,
            Surface::Overlay => self.overlay,
            Surface::Field => self.field,
        }
    }
}

/// Foregrounds grouped by content emphasis.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
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
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
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

/// Partially translucent color overlays to put over a base color for interaction states.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
pub struct Interactions {
    /// Overlay applied while a pointer hovers over an interactive region.
    pub hover: Color,
    /// Overlay applied while an interactive region is pressed.
    pub pressed: Color,
    /// Persistent overlay for selected content.
    pub selected: Color,
    /// Hover overlay for selected content.
    pub selected_hover: Color,
    /// Overlay used to highlight selected text in an editable field.
    pub text_selection: Color,
    /// Overlay for content currently being dragged.
    pub dragged: Color,
    /// Overlay identifying a valid drop destination.
    pub drop_target: Color,
}

impl Interactions {
    /// Returns the resolved overlay for an interaction state.
    pub const fn get(&self, state: Interaction) -> Color {
        match state {
            Interaction::Hover => self.hover,
            Interaction::Pressed => self.pressed,
            Interaction::Selected => self.selected,
            Interaction::SelectedHover => self.selected_hover,
            Interaction::Selection => self.text_selection,
            Interaction::Dragged => self.dragged,
            Interaction::DropTarget => self.drop_target,
        }
    }
}

/// A fill and the text or icon color guaranteed against it.
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
pub struct Pair {
    /// The background or fill color.
    pub color: Color,
    /// Text and icon color guaranteed against [`Self::color`].
    pub text: Color,
}

/// Resolved fill pairs for each interactive state.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
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
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
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

/// Syntax highlighting foregrounds for code shown on opaque neutral surfaces.
///
/// Hued roles keep their semantic seed's hue with boosted chroma. Unlike
/// semantic foregrounds, they are not validated over interaction overlays,
/// which leaves room for more saturated colors.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
pub struct Syntax {
    /// Keywords and storage modifiers, derived from the accent seed.
    pub keyword: Color,
    /// Types, classes, and paths, derived from the warning seed.
    pub type_name: Color,
    /// Functions and built-ins, derived from the accent seed with a rotated hue.
    pub function: Color,
    /// String and character literals, derived from the success seed.
    pub string: Color,
    /// Numeric, boolean, and other constants, derived from the danger seed.
    pub constant: Color,
    /// Comments, validated against the same surfaces as the hued roles.
    pub comment: Color,
}

impl Syntax {
    /// Gets the matching color for the given [`SyntaxRole`].
    pub fn color(&self, role: SyntaxRole) -> Color {
        match role {
            SyntaxRole::Keyword => self.keyword,
            SyntaxRole::TypeName => self.type_name,
            SyntaxRole::Function => self.function,
            SyntaxRole::String => self.string,
            SyntaxRole::Constant => self.constant,
            SyntaxRole::Comment => self.comment,
        }
    }
}

/// All resolved color tokens for a theme.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Animate)]
pub struct Colors {
    /// Neutral planes used throughout the interface.
    pub surfaces: Surfaces,
    /// Foregrounds grouped by content emphasis.
    pub content: Content,
    /// Decorative, control, focus, and selection boundaries.
    pub borders: Borders,
    /// Temporary and persistent interaction overlays.
    pub interaction: Interactions,
    /// Accent and primary-action roles.
    pub accent: Semantic,
    /// Successful and positive-state roles.
    pub success: Semantic,
    /// Caution and warning-state roles.
    pub warning: Semantic,
    /// Destructive, invalid, and error-state roles.
    pub danger: Semantic,
    /// Colors for distinguishing unordered categories.
    pub categorical: Categorical,
    /// Syntax highlighting foregrounds for code.
    pub syntax: Syntax,
}

impl Colors {
    /// Gets the matching [`Semantic`] colors for the given [`Intent`]
    pub fn semantic(&self, intent: Intent) -> Semantic {
        match intent {
            Intent::Accent => self.accent,
            Intent::Success => self.success,
            Intent::Warning => self.warning,
            Intent::Danger => self.danger,
        }
    }
}
