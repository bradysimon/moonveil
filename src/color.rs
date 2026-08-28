#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Srgba {
    /// The red component of the color, [0.0, 1.0].
    red: f32,
    /// The green component of the color, [0.0, 1.0].
    green: f32,
    /// The blue component of the color, [0.0, 1.0].
    blue: f32,
    /// The alpha component of the color, [0.0, 1.0].
    alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Oklaba {
    /// The lightness component of the color, [0.0, 1.0].
    lightness: f32,
    /// The 'a' component of the color, [-1.0, 1.0].
    a: f32,
    /// The 'b' component of the color, [-1.0, 1.0].
    b: f32,
    /// The alpha component of the color, [0.0, 1.0].
    alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Oklcha {
    /// The lightness component of the color, [0.0, 1.0].
    lightness: f32,
    /// The chroma component of the color, [0.0, 1.0].
    chroma: f32,
    /// The hue component of the color, [0.0, 360.0].
    hue: f32,
    /// The alpha component of the color, [0.0, 1.0].
    alpha: f32,
}
