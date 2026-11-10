//! A variety of different themes that you can use in the gallery.
use std::sync::LazyLock;

use moonveil::{CategoricalSeeds, Contrast, Definition, Metadata, Polarity, Seed, Theme, color};

const EVERFOREST_SEED: Seed = Seed {
    background: color!(0xf3f1e5),
    foreground: color!(0x5c6a72),
    tint: color!(0xfffff4),
    shade: color!(0x7f8c70),
    accent: color!(0x35a77c),
    success: color!(0x8da101),
    warning: color!(0xdfa000),
    danger: color!(0xf85552),
};

/// Everforest's seven accent hues; the eighth slot is generated.
const EVERFOREST_CATEGORICAL: CategoricalSeeds = CategoricalSeeds::new([
    color!(0x3a94c5), // blue
    color!(0xf57d26), // orange
    color!(0x8da101), // green
    color!(0xdf69ba), // purple
    color!(0x35a77c), // aqua
    color!(0xdfa000), // yellow
    color!(0xf85552), // red
]);

const ROSE_PINE_SEED: Seed = Seed {
    background: color!(0x191724),
    foreground: color!(0xe0def4),
    tint: color!(0xf4f1ff),
    shade: color!(0x0b0a12),
    accent: color!(0xc4a7e7),
    success: color!(0x9ccfd8),
    warning: color!(0xf6c177),
    danger: color!(0xeb6f92),
};

/// Rosé Pine's six accent hues; the last two slots are generated.
const ROSE_PINE_CATEGORICAL: CategoricalSeeds = CategoricalSeeds::new([
    color!(0xc4a7e7), // iris
    color!(0x9ccfd8), // foam
    color!(0xf6c177), // gold
    color!(0xeb6f92), // love
    color!(0xebbcba), // rose
    color!(0x31748f), // pine
]);

const CATPPUCCIN_LATTE_SEED: Seed = Seed {
    background: color!(0xeff1f5),
    foreground: color!(0x4c4f69),
    tint: color!(0xffffff),
    shade: color!(0x9ca0b0),
    accent: color!(0x1e66f5),
    success: color!(0x40a02b),
    warning: color!(0xdf8e1d),
    danger: color!(0xd20f39),
};

/// Eight of Catppuccin Latte's fourteen accent colors.
const CATPPUCCIN_LATTE_CATEGORICAL: CategoricalSeeds = CategoricalSeeds::new([
    color!(0x1e66f5), // blue
    color!(0xfe640b), // peach
    color!(0x40a02b), // green
    color!(0x8839ef), // mauve
    color!(0x179299), // teal
    color!(0xea76cb), // pink
    color!(0xdf8e1d), // yellow
    color!(0xd20f39), // red
]);

const NORD_SEED: Seed = Seed {
    background: color!(0x2e3440),
    foreground: color!(0xd8dee9),
    tint: color!(0xeceff4),
    shade: color!(0x1b1f27),
    accent: color!(0x88c0d0),
    success: color!(0xa3be8c),
    warning: color!(0xebcb8b),
    danger: color!(0xbf616a),
};

/// Nord's Frost and Aurora colors.
const NORD_CATEGORICAL: CategoricalSeeds = CategoricalSeeds::new([
    color!(0x88c0d0), // nord8
    color!(0xd08770), // nord12
    color!(0xa3be8c), // nord14
    color!(0xb48ead), // nord15
    color!(0xebcb8b), // nord13
    color!(0x5e81ac), // nord10
    color!(0x8fbcbb), // nord7
    color!(0xbf616a), // nord11
]);

static EVERFOREST_THEME: LazyLock<Theme> = LazyLock::new(|| {
    resolve(
        "Everforest Light",
        "A soft paper palette with moss green, aqua, and warm woodland accents.",
        Polarity::Light,
        EVERFOREST_SEED,
        EVERFOREST_CATEGORICAL,
    )
});

static ROSE_PINE_THEME: LazyLock<Theme> = LazyLock::new(|| {
    resolve(
        "Rose Pine",
        "A softened black base with restrained rose, foam, gold, and iris accents.",
        Polarity::Dark,
        ROSE_PINE_SEED,
        ROSE_PINE_CATEGORICAL,
    )
});

static CATPPUCCIN_LATTE_THEME: LazyLock<Theme> = LazyLock::new(|| {
    resolve(
        "Catppuccin Latte",
        "A neutral light workspace with crisp blue action color and saturated semantics.",
        Polarity::Light,
        CATPPUCCIN_LATTE_SEED,
        CATPPUCCIN_LATTE_CATEGORICAL,
    )
});

static NORD_THEME: LazyLock<Theme> = LazyLock::new(|| {
    resolve(
        "Nord",
        "A glacial blue-gray palette with frost accents and quiet arctic semantics.",
        Polarity::Dark,
        NORD_SEED,
        NORD_CATEGORICAL,
    )
});

pub fn everforest() -> Theme {
    LazyLock::force(&EVERFOREST_THEME).clone()
}

pub fn rose_pine() -> Theme {
    LazyLock::force(&ROSE_PINE_THEME).clone()
}

pub fn catppuccin_latte() -> Theme {
    LazyLock::force(&CATPPUCCIN_LATTE_THEME).clone()
}

pub fn nord() -> Theme {
    LazyLock::force(&NORD_THEME).clone()
}

fn resolve(
    name: &'static str,
    description: &'static str,
    polarity: Polarity,
    seed: Seed,
    categorical: CategoricalSeeds,
) -> Theme {
    Theme::new(Definition {
        categorical,
        ..Definition::new(
            Metadata {
                name: name.into(),
                description: Some(description.into()),
            },
            polarity,
            Contrast::Standard,
            seed,
        )
    })
    .expect("gallery preview theme definitions must resolve")
}

#[cfg(test)]
mod tests {
    use super::{catppuccin_latte, everforest, nord, rose_pine};

    #[test]
    fn all_preview_themes_resolve() {
        let _ = [everforest(), rose_pine(), catppuccin_latte(), nord()];
    }
}
