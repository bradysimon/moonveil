//! Syntax highlighting colored by Moonveil's resolved theme tokens.
//!
//! [`highlight`] maps each [`Code`] region onto the theme's syntax and content
//! tokens, which are validated for text contrast against every opaque neutral
//! surface. Highlighted code stays readable on any Moonveil plane.
//!
//! Enable the `highlighter` feature to parse code with [`Parser`] for
//! Markdown code blocks and text editors.

use crate::Theme;
use iced_core::font;

pub use iced_core::Code;
pub use iced_core::text::highlighter::{Highlighter, Style};
#[cfg(feature = "highlighter")]
pub use iced_highlighter::{Parser, Settings, Stream};

/// Returns the highlight [`Style`] of a [`Code`] region for the given [`Theme`].
pub fn highlight(code: Code, theme: &Theme) -> Style {
    let colors = theme.colors();
    let syntax = colors.syntax;

    let color = match code {
        Code::Keyword => Some(syntax.keyword),
        Code::Type | Code::Path => Some(syntax.type_name),
        Code::Function | Code::Support => Some(syntax.function),
        Code::Constant => Some(syntax.constant),
        Code::String => Some(syntax.string),
        Code::Comment => Some(syntax.comment),
        Code::Invalid => Some(colors.danger.foreground),
        Code::Punctuation => Some(colors.content.secondary),
        Code::Variable | Code::Other => None,
    };

    Style {
        color: color.map(Into::into),
        style: (code == Code::Comment).then_some(font::Style::Italic),
    }
}
