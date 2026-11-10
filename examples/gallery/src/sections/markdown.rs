use crate::gallery::Gallery;
use crate::message::Message;
use crate::ui::{band, page, panel, section_heading, PANEL_PADDING};
use iced::Length::Fill;
use iced::widget::column;
use moonveil::widget::text::TextExt as _;
use moonveil::widget::text_editor::TextEditorExt as _;
use moonveil::widget::{container, markdown, scrollable, text, text_editor, Row};
use moonveil::{highlighter, Element, spacing};

pub(crate) const MARKDOWN_SAMPLE: &str = r#"# Release notes

Moonveil renders **Markdown** with the active theme. Links such as
[the Iced repository](https://github.com/iced-rs/iced) use the accent token,
and `inline code` sits in a sunken well.

## Highlights

- Syntax highlighting colored by theme tokens
- Quotes, tables, and task lists
- Styles that follow theme and rounding changes

> Code colors come from contrast-validated foreground tokens, so they stay
> readable on every surface.

```rust
/// Promotes a release once every check has passed.
fn promote(release: &Release, target: Environment) -> Result<(), Error> {
    let failed = release.checks().filter(|check| !check.passed()).count();

    if failed > 0 {
        return Err(Error::FailedChecks(failed));
    }

    deploy(release, target, "rolling")
}
```

```toml
[deploy]
region = "us-east-1"
replicas = 3
canary = true
```

### Checklist

- [x] Resolve theme tokens
- [ ] Promote to production

| Environment | Region | Status |
| :--- | :--- | :---: |
| Production | us-east-1 | Healthy |
| Staging | eu-west-1 | Rolling |
"#;

pub(super) fn markdown_section(gallery: &Gallery) -> Element<'_, Message> {
    page([
        panel(section_heading(
            "DOCUMENT CATALOGS",
            "Markdown and syntax highlighting",
            Some("Edit the highlighted source to update the rendered preview. Code colors resolve from the active theme's syntax tokens."),
        ))
        .into(),
        band([markdown_source(gallery), markdown_preview(gallery)]),
        syntax_legend(),
    ])
}

fn markdown_source(gallery: &Gallery) -> Element<'_, Message> {
    panel(
        column![
            text("Source").class(text::Variant::Primary),
            text("Text editor highlighted as Markdown")
                .size(text::Size::Label)
                .class(text::Variant::Muted),
            text_editor(&gallery.markdown_source)
                .on_action(Message::EditMarkdown)
                .class(text_editor::Variant::Code)
                .font(iced::Font::MONOSPACE)
                .size(text::Size::Label)
                .height(520)
                .padding(spacing::MD)
                .highlight("md"),
        ]
        .spacing(9),
    )
    .into()
}

fn markdown_preview(gallery: &Gallery) -> Element<'_, Message> {
    let preview = markdown::view(
        gallery.markdown.items(),
        markdown::settings(),
        gallery.theme.clone(),
    )
    .map(Message::OpenLink);

    panel(
        column![
            text("Preview").class(text::Variant::Primary),
            text("Rendered with the Moonveil markdown catalog")
                .size(text::Size::Label)
                .class(text::Variant::Muted),
            scrollable(container(preview).padding(iced::Padding {
                right: spacing::MD,
                ..iced::Padding::ZERO
            }))
            .height(520),
        ]
        .spacing(9),
    )
    .into()
}

fn syntax_legend() -> Element<'static, Message> {
    let tokens = [
        ("keyword", highlighter::Code::Keyword),
        ("Type", highlighter::Code::Type),
        ("function", highlighter::Code::Function),
        ("\"string\"", highlighter::Code::String),
        ("42", highlighter::Code::Constant),
        ("// comment", highlighter::Code::Comment),
        ("{ }", highlighter::Code::Punctuation),
    ];

    let swatches =
        tokens
            .into_iter()
            .fold(Row::new().spacing(spacing::LG), |row, (sample, code)| {
                row.push(
                    text(sample)
                        .font(iced::Font {
                            style: if code == highlighter::Code::Comment {
                                iced::font::Style::Italic
                            } else {
                                iced::font::Style::Normal
                            },
                            ..iced::Font::MONOSPACE
                        })
                        .size(text::Size::Label)
                        .style(move |theme| text::Style {
                            color: highlighter::highlight(code, theme).color,
                        }),
                )
            });

    container(
        column![
            text("SYNTAX TOKENS")
                .size(text::Size::Label)
                .class(text::Variant::Muted),
            swatches,
        ]
        .spacing(spacing::SM),
    )
    .class(container::Variant::Sunken)
    .padding(PANEL_PADDING)
    .width(Fill)
    .into()
}
