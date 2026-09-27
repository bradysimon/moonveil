//! Markdown rendering styled by Moonveil's concrete theme.
//!
//! Parse Markdown once with [`parse`] or [`Content`], then display the items
//! with [`view`] using [`settings`] for Moonveil's type scale. Enable the
//! `highlighter` feature to color fenced code blocks with
//! [`highlighter::highlight`](crate::highlighter::highlight).

use crate::{Theme, highlighter, spacing, widget::container, widget::text};
use iced_core::{Background, Border, Color, Padding, Pixels};

pub use iced_widget::markdown::{
    Bullet, Catalog, Column, Content, DefaultViewer, HeadingLevel, Highlight, InlineCode, Item,
    Row, Settings, Text, Uri, Viewer, code_block, heading, item, items, ordered_list, paragraph,
    parse, quote, rule, sections, table, unordered_list, view, view_with,
};

/// Returns Markdown [`Settings`] aligned with Moonveil's type scale and spacing.
pub fn settings() -> Settings {
    Settings {
        inline_code_size: Pixels(text::size::LABEL),
        code_block_size: Pixels(text::size::LABEL),
        h1_size: Pixels(text::size::DISPLAY),
        h2_size: Pixels(text::size::HEADING),
        h3_size: Pixels(text::size::TITLE),
        spacing: Pixels(spacing::MD),
        ..Settings::with_text_size(text::size::BODY)
    }
}

impl Catalog for Theme {
    fn id(&self) -> &str {
        Theme::id(self)
    }

    fn link_color(&self) -> Color {
        self.colors().accent.foreground.into()
    }

    fn code(&self) -> InlineCode {
        InlineCode {
            padding: Padding::from([1.0, spacing::XS]),
            highlight: Highlight {
                background: Background::Color(self.colors().surfaces.sunken.into()),
                border: Border {
                    color: self.colors().borders.subtle.into(),
                    width: self.appearance().border.hairline,
                    radius: self.appearance().radius.xs.into(),
                },
            },
            color: self.colors().content.primary.into(),
        }
    }

    fn code_block<'a>() -> container::Class<'a> {
        container::Variant::Sunken.into()
    }

    fn quote<'a>() -> container::Class<'a> {
        container::Variant::Inset.into()
    }

    fn highlighter(&self) -> &dyn highlighter::Highlighter<highlighter::Code, Self> {
        &highlighter::highlight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_id_tracks_theme_resolution() {
        let theme = Theme::default_dark();
        let rebuilt = Theme::new(theme.definition().clone()).unwrap();

        assert_eq!(Catalog::id(&theme), theme.id());
        assert_ne!(Catalog::id(&theme), Catalog::id(&rebuilt));
    }

    #[test]
    fn links_and_inline_code_use_theme_tokens() {
        let theme = Theme::default_light();
        let code = theme.code();

        assert_eq!(theme.link_color(), theme.colors().accent.foreground.into());
        assert_eq!(code.color, theme.colors().content.primary.into());
        assert_eq!(
            code.highlight.background,
            Background::Color(theme.colors().surfaces.sunken.into())
        );
    }

    #[test]
    fn settings_follow_the_type_scale() {
        let settings = settings();

        assert_eq!(settings.text_size, Pixels(text::size::BODY));
        assert_eq!(settings.h1_size, Pixels(text::size::DISPLAY));
        assert_eq!(settings.h4_size, Pixels(text::size::BODY));
    }
}
