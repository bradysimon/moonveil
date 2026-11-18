//! Only rebuild content when its dependency changes.

pub type Lazy<'a, W, Dependency> = iced_widget::Lazy<'a, W, Dependency>;

/// Creates a [`Lazy`] widget that rebuilds `view` only when `dependency`'s hash changes.
pub fn lazy<'a, Dependency, View>(
    dependency: Dependency,
    view: impl Fn(&Dependency) -> View + 'a,
) -> Lazy<'a, View, Dependency>
where
    Dependency: std::hash::Hash + 'a,
{
    Lazy::new(dependency, view)
}
