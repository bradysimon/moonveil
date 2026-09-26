use std::sync::LazyLock;

use moonveil::{Color, Contrast, Definition, Metadata, Polarity, Seed, Theme};

const EVERFOREST_SEED: Seed = Seed {
    background: Color::from_rgb8(0xf3, 0xf1, 0xe5),
    foreground: Color::from_rgb8(0x5c, 0x6a, 0x72),
    tint: Color::from_rgb8(0xff, 0xff, 0xf4),
    shade: Color::from_rgb8(0x7f, 0x8c, 0x70),
    accent: Color::from_rgb8(0x35, 0xa7, 0x7c),
    success: Color::from_rgb8(0x8d, 0xa1, 0x01),
    warning: Color::from_rgb8(0xdf, 0xa0, 0x00),
    danger: Color::from_rgb8(0xf8, 0x55, 0x52),
    info: Color::from_rgb8(0x3a, 0x94, 0xc5),
};

const ROSE_PINE_SEED: Seed = Seed {
    background: Color::from_rgb8(0x19, 0x17, 0x24),
    foreground: Color::from_rgb8(0xe0, 0xde, 0xf4),
    tint: Color::from_rgb8(0xf4, 0xf1, 0xff),
    shade: Color::from_rgb8(0x0b, 0x0a, 0x12),
    accent: Color::from_rgb8(0xc4, 0xa7, 0xe7),
    success: Color::from_rgb8(0x9c, 0xcf, 0xd8),
    warning: Color::from_rgb8(0xf6, 0xc1, 0x77),
    danger: Color::from_rgb8(0xeb, 0x6f, 0x92),
    info: Color::from_rgb8(0x8b, 0xd5, 0xca),
};

const CATPPUCCIN_LATTE_SEED: Seed = Seed {
    background: Color::from_rgb8(0xef, 0xf1, 0xf5),
    foreground: Color::from_rgb8(0x4c, 0x4f, 0x69),
    tint: Color::from_rgb8(0xff, 0xff, 0xff),
    shade: Color::from_rgb8(0x9c, 0xa0, 0xb0),
    accent: Color::from_rgb8(0x1e, 0x66, 0xf5),
    success: Color::from_rgb8(0x40, 0xa0, 0x2b),
    warning: Color::from_rgb8(0xdf, 0x8e, 0x1d),
    danger: Color::from_rgb8(0xd2, 0x0f, 0x39),
    info: Color::from_rgb8(0x0f, 0x7f, 0x88),
};

const NORD_SEED: Seed = Seed {
    background: Color::from_rgb8(0x2e, 0x34, 0x40),
    foreground: Color::from_rgb8(0xd8, 0xde, 0xe9),
    tint: Color::from_rgb8(0xec, 0xef, 0xf4),
    shade: Color::from_rgb8(0x1b, 0x1f, 0x27),
    accent: Color::from_rgb8(0x88, 0xc0, 0xd0),
    success: Color::from_rgb8(0xa3, 0xbe, 0x8c),
    warning: Color::from_rgb8(0xeb, 0xcb, 0x8b),
    danger: Color::from_rgb8(0xbf, 0x61, 0x6a),
    info: Color::from_rgb8(0x8f, 0xbc, 0xbb),
};

static EVERFOREST_THEME: LazyLock<Theme> = LazyLock::new(|| {
    resolve(
        "Everforest Light",
        "A soft paper palette with moss green, aqua, and warm woodland accents.",
        Polarity::Light,
        EVERFOREST_SEED,
    )
});

static ROSE_PINE_THEME: LazyLock<Theme> = LazyLock::new(|| {
    resolve(
        "Rose Pine",
        "A softened black base with restrained rose, foam, gold, and iris accents.",
        Polarity::Dark,
        ROSE_PINE_SEED,
    )
});

static CATPPUCCIN_LATTE_THEME: LazyLock<Theme> = LazyLock::new(|| {
    resolve(
        "Catppuccin Latte",
        "A neutral light workspace with crisp blue action color and saturated semantics.",
        Polarity::Light,
        CATPPUCCIN_LATTE_SEED,
    )
});

static NORD_THEME: LazyLock<Theme> = LazyLock::new(|| {
    resolve(
        "Nord",
        "A glacial blue-gray palette with frost accents and quiet arctic semantics.",
        Polarity::Dark,
        NORD_SEED,
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

fn resolve(name: &'static str, description: &'static str, polarity: Polarity, seed: Seed) -> Theme {
    Theme::new(Definition::new(
        Metadata {
            name: name.into(),
            description: Some(description.into()),
        },
        polarity,
        Contrast::Standard,
        seed,
    ))
    .expect("gallery preview theme definitions must resolve")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_preview_themes_resolve() {
        let _ = [everforest(), rose_pine(), catppuccin_latte(), nord()];
    }
}
