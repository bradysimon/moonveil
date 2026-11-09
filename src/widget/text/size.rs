//! Various text sizes that you can use.
//!
//! Choose a [`Size`] and then optionally a [`Scale`] using one of the following
//! to adjust the final text size:
//!
//! - [`Size::sm`]
//! - [`Size::md`]
//! - [`Size::lg`]

use iced_core::Pixels;

/// A pre-defined typography role for text.
///
/// The variants describe how text is used. Choose a scale with [`Size::sm`],
/// [`Size::md`], or [`Size::lg`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Size {
    /// Large, prominent text. Typically reserved for short, important content or large numerals.
    Display,
    /// Prominent text for short, important content, below display in hierarchy.
    Headline,
    /// Text for titles and headings that establish hierarchy within content.
    Title,
    /// General text suited to longer passages and supporting content.
    #[default]
    Body,
    /// Compact helper text for component labels and other brief content.
    Label,
}

/// Text scaling options for adjusting the final text size.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Scale {
    /// Adjusts the size of the text to be smaller than the default.
    Small,
    /// The default text size scaling.
    #[default]
    Medium,
    /// Adjusts the size of the text to be larger than the default.
    Large,
}

impl Size {
    /// Returns the text size in pixels for the given `scale`.
    pub const fn px(&self, scale: Scale) -> Pixels {
        match scale {
            Scale::Small => Pixels(self.sm()),
            Scale::Medium => Pixels(self.md()),
            Scale::Large => Pixels(self.lg()),
        }
    }

    /// Returns the small size for the text variant.
    pub const fn sm(&self) -> f32 {
        match self {
            Size::Display => 36.0,
            Size::Headline => 24.0,
            Size::Title => 14.0,
            Size::Body => 12.0,
            Size::Label => 11.0,
        }
    }

    /// Returns the medium (default) size for the text variant.
    pub const fn md(&self) -> f32 {
        match self {
            Size::Display => 45.0,
            Size::Headline => 28.0,
            Size::Title => 16.0,
            Size::Body => 14.0,
            Size::Label => 12.0,
        }
    }

    /// Returns the large size for the text variant.
    pub const fn lg(&self) -> f32 {
        match self {
            Size::Display => 56.0,
            Size::Headline => 32.0,
            Size::Title => 22.0,
            Size::Body => 16.0,
            Size::Label => 14.0,
        }
    }
}

impl From<Size> for f32 {
    fn from(size: Size) -> Self {
        size.md()
    }
}

impl From<Size> for Pixels {
    fn from(size: Size) -> Self {
        size.px(Scale::default())
    }
}
