//! Apply a different Moonveil theme to a subtree.

use crate::{Element, Theme};

pub type Themer<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Themer<'a, Message, Theme, Renderer>;

/// Renders `content` with `theme`, or the inherited theme when `None`.
pub fn themer<'a, Message, Renderer>(
    theme: Option<Theme>,
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Themer<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    Themer::new(theme, content)
}
