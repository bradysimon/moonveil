//! Only rebuild content when its dependency changes.

use crate::{Element, Theme};

pub type Lazy<'a, Message, Dependency, View, Renderer = iced_widget::Renderer> =
    iced_widget::Lazy<'a, Message, Theme, Renderer, Dependency, View>;

/// Creates a [`Lazy`] widget that rebuilds `view` only when `dependency`'s hash changes.
pub fn lazy<'a, Message, Renderer, Dependency, View>(
    dependency: Dependency,
    view: impl Fn(&Dependency) -> View + 'a,
) -> Lazy<'a, Message, Dependency, View, Renderer>
where
    Dependency: std::hash::Hash + 'a,
    View: Into<Element<'static, Message, Renderer>>,
{
    Lazy::new(dependency, view)
}
