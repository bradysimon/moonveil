/// Represents the different sections in the gallery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Section {
    Overview,
    Actions,
    Indicators,
    Selection,
    Fields,
    Choice,
    Surfaces,
    Overlays,
    Markdown,
}

impl Section {
    pub(super) const ALL: [Self; 9] = [
        Self::Overview,
        Self::Actions,
        Self::Indicators,
        Self::Selection,
        Self::Fields,
        Self::Choice,
        Self::Surfaces,
        Self::Overlays,
        Self::Markdown,
    ];

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Actions => "Actions",
            Self::Indicators => "Badges & progress",
            Self::Selection => "Controls & selection",
            Self::Fields => "Fields & editor",
            Self::Choice => "Choice & scrolling",
            Self::Surfaces => "Surfaces & type",
            Self::Overlays => "Overlays & menus",
            Self::Markdown => "Markdown & code",
        }
    }
}
