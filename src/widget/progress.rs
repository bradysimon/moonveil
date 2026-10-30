//! Indeterminate progress indicators for work of unknown duration.
//!
//! Use [`ProgressBar`](super::ProgressBar) when progress can be measured.

use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::sync::OnceLock;

use crate::{Element, Theme, token::Intent};
use iced_core::{
    Color, Event, Layout, Length, Point, Rectangle, Shell, Vector, Widget, layout, mouse, renderer,
    time::Instant,
    widget::{Tree, tree},
    window,
};
use iced_widget::canvas::{Frame, LineCap, Path, Stroke, path::Arc};
use iced_widget::graphics::geometry;

/// The track opacity used when a style derives its track from the indicator.
const TRACK_ALPHA: f32 = 0.2;

/// Creates an indeterminate progress indicator.
pub fn progress<'a>() -> Progress<'a> {
    Progress::new()
}

/// An animated indicator for work of unknown duration.
pub struct Progress<'a> {
    shape: Shape,
    size: Size,
    width: Option<Length>,
    class: Class<'a>,
}

impl<'a> Progress<'a> {
    /// Creates an indeterminate progress indicator.
    pub fn new() -> Self {
        Self {
            shape: Shape::default(),
            size: Size::default(),
            width: None,
            class: Class::default(),
        }
    }

    /// Sets the drawn form and motion of the indicator.
    #[must_use]
    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }

    /// Sets the scale of the indicator.
    #[must_use]
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Sets the width of the indicator; compact shapes are centered within it.
    ///
    /// [`Shape::Bar`] fills the available width by default.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Sets the style of the indicator.
    #[must_use]
    pub fn class(mut self, class: impl Into<Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets a custom style function.
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self {
        self.class = Class::Custom(Box::new(style));
        self
    }

    fn dimensions(&self) -> iced_core::Size<Length> {
        let diameter = self.size.diameter();
        let (width, height) = match self.shape {
            Shape::Bar => (Length::Fill, self.size.bar_thickness()),
            Shape::Dots => (Length::Fixed(diameter), diameter / 2.0),
            Shape::Arc
            | Shape::Sweep
            | Shape::Spokes
            | Shape::Orbit
            | Shape::Bars
            | Shape::Pulse => (Length::Fixed(diameter), diameter),
        };

        iced_core::Size::new(self.width.unwrap_or(width), Length::Fixed(height))
    }
}

impl Default for Progress<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// The drawn form and motion of an indeterminate indicator.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Shape {
    /// A quarter arc circling a faint track; the general-purpose default.
    #[default]
    Arc,
    /// A trackless arc that grows and shrinks as it rotates.
    Sweep,
    /// Radial spokes fading in sequence, like a system activity indicator.
    Spokes,
    /// A ring of dots with a bright head chasing around it.
    Orbit,
    /// Three dots bouncing in turn; suited to inline "typing" or chat states.
    Dots,
    /// Vertical bars rising and falling in a wave; suited to streaming or audio.
    Bars,
    /// A solid core emitting fading ripples; suited to live or syncing status.
    Pulse,
    /// A segment sliding along a full-width track.
    Bar,
}

impl Shape {
    /// Every built-in shape, in declaration order.
    pub const ALL: [Self; 8] = [
        Self::Arc,
        Self::Sweep,
        Self::Spokes,
        Self::Orbit,
        Self::Dots,
        Self::Bars,
        Self::Pulse,
        Self::Bar,
    ];
}

/// The scale of an indeterminate indicator.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Size {
    /// Fits beside control labels and inside buttons.
    Small,
    #[default]
    Medium,
    /// Suited to empty panels and full-page loading states.
    Large,
}

impl Size {
    /// The diameter of circular shapes in logical pixels.
    pub const fn diameter(self) -> f32 {
        match self {
            Self::Small => 16.0,
            Self::Medium => 24.0,
            Self::Large => 32.0,
        }
    }

    /// The stroke width of arcs and spokes.
    pub const fn stroke(self) -> f32 {
        match self {
            Self::Small => 2.0,
            Self::Medium => 2.5,
            Self::Large => 3.0,
        }
    }

    /// The height of [`Shape::Bar`].
    pub const fn bar_thickness(self) -> f32 {
        match self {
            Self::Small => 2.0,
            Self::Medium => 4.0,
            Self::Large => 6.0,
        }
    }
}

/// The resolved colors of an indeterminate indicator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The moving mark; `None` inherits the surrounding text color.
    pub indicator: Option<Color>,
    /// The static track behind arcs and bars; `None` derives it from the indicator.
    pub track: Option<Color>,
}

/// A custom progress style function using Moonveil's concrete theme.
pub type StyleFn<'a> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// A built-in Moonveil progress style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// The accent indicator.
    #[default]
    Standard,
    /// A low-emphasis neutral indicator.
    Neutral,
    /// Follows the surrounding text color, such as a button label.
    Inherited,
    /// An indicator carrying explicit semantic intent.
    Semantic(Intent),
}

/// A built-in variant or downstream custom style function.
pub enum Class<'a> {
    /// A built-in, allocation-free Moonveil variant.
    Variant(Variant),
    /// A downstream custom style function.
    Custom(StyleFn<'a>),
}

impl Default for Class<'_> {
    fn default() -> Self {
        Self::Variant(Variant::default())
    }
}

impl<'a> From<Variant> for Class<'a> {
    fn from(variant: Variant) -> Self {
        Self::Variant(variant)
    }
}

impl<'a> From<StyleFn<'a>> for Class<'a> {
    fn from(style: StyleFn<'a>) -> Self {
        Self::Custom(style)
    }
}

/// The theme catalog used to style an indeterminate indicator.
pub trait Catalog {
    /// The progress style class type.
    type Class<'a>;

    /// Returns the default progress class.
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a progress class into a style.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = Class<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Class::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        match class {
            Class::Variant(variant) => appearance(self, *variant),
            Class::Custom(style) => style(self),
        }
    }
}

/// Returns the resolved style for a built-in progress variant.
pub fn appearance(theme: &Theme, variant: Variant) -> Style {
    let colors = theme.colors();
    let track = Some(colors.interaction.pressed.into());

    match variant {
        Variant::Standard => Style {
            indicator: Some(colors.accent.indicator.into()),
            track,
        },
        Variant::Neutral => Style {
            indicator: Some(colors.content.secondary.into()),
            track,
        },
        Variant::Inherited => Style {
            indicator: None,
            track: None,
        },
        Variant::Semantic(intent) => Style {
            indicator: Some(colors.semantic(intent).indicator.into()),
            track,
        },
    }
}

struct State {
    now: Instant,
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Progress<'_>
where
    Renderer: geometry::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            now: Instant::now(),
        })
    }

    fn size(&self) -> iced_core::Size<Length> {
        self.dimensions()
    }

    fn layout(&mut self, tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) {
        let size = self.dimensions();
        tree.size = layout::atomic(limits, size.width, size.height);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            tree.state.downcast_mut::<State>().now = *now;

            // Scrolling back into view triggers a redraw, which resumes the animation.
            if layout.bounds().intersection(viewport).is_some() {
                shell.request_redraw();
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if bounds.intersection(viewport).is_none() {
            return;
        }

        let resolved = Catalog::style(theme, &self.class);
        let indicator = resolved.indicator.unwrap_or(style.text_color);
        let colors = Colors {
            indicator,
            track: resolved
                .track
                .unwrap_or_else(|| fade(indicator, TRACK_ALPHA)),
        };
        let time = elapsed(tree.state.downcast_ref::<State>().now);

        let geometry = Painter {
            frame: Frame::new(renderer, bounds.size()),
            colors,
            size: self.size,
            time,
        }
        .paint(self.shape)
        .into_geometry();

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            renderer.draw_geometry(geometry);
        });
    }
}

impl<'a, Message, Renderer> From<Progress<'a>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: geometry::Renderer + 'a,
{
    fn from(progress: Progress<'a>) -> Self {
        Element::new(progress)
    }
}

#[derive(Clone, Copy)]
struct Colors {
    indicator: Color,
    track: Color,
}

struct Painter<Renderer: geometry::Renderer> {
    frame: Frame<Renderer>,
    colors: Colors,
    size: Size,
    time: f64,
}

impl<Renderer: geometry::Renderer> Painter<Renderer> {
    fn paint(mut self, shape: Shape) -> Frame<Renderer> {
        match shape {
            Shape::Arc => self.arc(),
            Shape::Sweep => self.sweep(),
            Shape::Spokes => self.spokes(),
            Shape::Orbit => self.orbit(),
            Shape::Dots => self.dots(),
            Shape::Bars => self.bars(),
            Shape::Pulse => self.pulse(),
            Shape::Bar => self.bar(),
        }

        self.frame
    }

    fn center(&self) -> Point {
        self.frame.center()
    }

    /// The radius of the largest circle centered in the frame.
    fn radius(&self) -> f32 {
        let size = self.frame.size();
        size.width.min(size.height) / 2.0
    }

    fn stroke_arc(&mut self, radius: f32, start: f32, end: f32, color: Color) {
        let center = self.center();
        let arc = Path::new(|builder| {
            builder.arc(Arc {
                center,
                radius,
                start_angle: start.into(),
                end_angle: end.into(),
            });
        });

        self.frame.stroke(
            &arc,
            Stroke::default()
                .with_color(color)
                .with_width(self.size.stroke())
                .with_line_cap(LineCap::Round),
        );
    }

    fn arc(&mut self) {
        let width = self.size.stroke();
        let radius = self.radius() - width / 2.0;

        self.frame.stroke(
            &Path::circle(self.center(), radius),
            Stroke::default()
                .with_color(self.colors.track)
                .with_width(width),
        );

        let start = turns(phase(self.time, 0.8));
        self.stroke_arc(radius, start, start + TAU * 0.25, self.colors.indicator);
    }

    fn sweep(&mut self) {
        let radius = self.radius() - self.size.stroke() / 2.0;
        let (start, end) = sweep_span(self.time);

        self.stroke_arc(radius, turns(start), turns(end), self.colors.indicator);
    }

    fn spokes(&mut self) {
        const COUNT: usize = 12;

        let width = self.size.stroke();
        let center = self.center();
        let outer = self.radius() - width / 2.0;
        let inner = self.radius() * 0.45;
        // Steps rather than glides, like a system activity indicator.
        let head = (phase(self.time, 1.0) * COUNT as f32) as usize;

        for index in 0..COUNT {
            let age = (head + COUNT - index) % COUNT;
            let alpha = 1.0 - 0.85 * age as f32 / COUNT as f32;
            let direction = unit(turns(index as f32 / COUNT as f32));

            self.frame.stroke(
                &Path::line(center + direction * inner, center + direction * outer),
                Stroke::default()
                    .with_color(fade(self.colors.indicator, alpha))
                    .with_width(width)
                    .with_line_cap(LineCap::Round),
            );
        }
    }

    fn orbit(&mut self) {
        const COUNT: usize = 8;

        let dot = self.radius() * 0.2;
        let orbit = self.radius() - dot;
        let center = self.center();
        let head = phase(self.time, 1.0);

        for index in 0..COUNT {
            let offset = index as f32 / COUNT as f32;
            let intensity = (1.0 - (head - offset).rem_euclid(1.0)).powi(2);
            let position = center + unit(turns(offset)) * orbit;

            self.frame.fill(
                &Path::circle(position, dot * (0.55 + 0.45 * intensity)),
                fade(self.colors.indicator, 0.2 + 0.8 * intensity),
            );
        }
    }

    fn dots(&mut self) {
        let diameter = self.size.diameter();
        let dot = diameter / 4.0;
        let gap = diameter / 8.0;
        let center = self.center();
        let left = center.x - diameter / 2.0 + dot / 2.0;
        let rest = center.y + diameter / 4.0 - dot / 2.0;
        let lift = diameter / 4.0;
        let cycle = phase(self.time, 1.2);

        for index in 0..3 {
            let local = (cycle - index as f32 * 0.15).rem_euclid(1.0);
            let bounce = if local < 0.4 {
                (local / 0.4 * PI).sin()
            } else {
                0.0
            };
            let position = Point::new(left + index as f32 * (dot + gap), rest - bounce * lift);

            self.frame.fill(
                &Path::circle(position, dot / 2.0),
                fade(self.colors.indicator, 0.45 + 0.55 * bounce),
            );
        }
    }

    fn bars(&mut self) {
        const COUNT: usize = 4;

        let diameter = self.size.diameter();
        let width = diameter / (2 * COUNT - 1) as f32;
        let center = self.center();
        let left = center.x - diameter / 2.0;
        let cycle = phase(self.time, 1.0);

        for index in 0..COUNT {
            let local = (cycle - index as f32 * 0.12).rem_euclid(1.0);
            let level = 0.5 - 0.5 * (local * TAU).cos();
            let height = diameter * (0.3 + 0.7 * level);
            let top_left = Point::new(left + index as f32 * width * 2.0, center.y - height / 2.0);

            self.frame.fill(
                &Path::rounded_rectangle(
                    top_left,
                    iced_core::Size::new(width, height),
                    (width / 2.0).into(),
                ),
                self.colors.indicator,
            );
        }
    }

    fn pulse(&mut self) {
        let radius = self.radius();
        let core = radius * 0.35;
        let center = self.center();
        let cycle = phase(self.time, 1.6);

        for ripple in [cycle, (cycle + 0.5).rem_euclid(1.0)] {
            self.frame.fill(
                &Path::circle(center, core + (radius - core) * ease_out(ripple)),
                fade(self.colors.indicator, 0.5 * (1.0 - ripple)),
            );
        }

        self.frame
            .fill(&Path::circle(center, core), self.colors.indicator);
    }

    fn bar(&mut self) {
        let size = self.frame.size();
        let radius = size.height / 2.0;

        self.frame.fill(
            &Path::rounded_rectangle(Point::ORIGIN, size, radius.into()),
            self.colors.track,
        );

        let segment = size.width * 0.35;
        let start = -segment + (size.width + segment) * ease_in_out(phase(self.time, 1.6));
        let left = start.max(0.0);
        let right = (start + segment).min(size.width);

        if right > left {
            let width = right - left;
            self.frame.fill(
                &Path::rounded_rectangle(
                    Point::new(left, 0.0),
                    iced_core::Size::new(width, size.height),
                    radius.min(width / 2.0).into(),
                ),
                self.colors.indicator,
            );
        }
    }
}

/// Seconds since the first indicator was drawn, shared so indicators animate in sync.
///
/// Ensures that no matter where you draw an indicator, it will animate in sync with others.
fn elapsed(now: Instant) -> f64 {
    static EPOCH: OnceLock<Instant> = OnceLock::new();

    let epoch = *EPOCH.get_or_init(|| now);
    now.saturating_duration_since(epoch).as_secs_f64()
}

/// Progress through a repeating `period` in `0.0..1.0`.
fn phase(time: f64, period: f64) -> f32 {
    (time.rem_euclid(period) / period) as f32
}

/// Converts clockwise turns from twelve o'clock into radians from the positive x-axis.
fn turns(turns: f32) -> f32 {
    turns * TAU - FRAC_PI_2
}

fn unit(angle: f32) -> Vector {
    let (sin, cos) = angle.sin_cos();
    Vector::new(cos, sin)
}

fn fade(color: Color, alpha: f32) -> Color {
    Color {
        a: color.a * alpha,
        ..color
    }
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// The start and end of [`Shape::Sweep`]'s arc, in turns.
fn sweep_span(time: f64) -> (f32, f32) {
    const CYCLE: f64 = 1.4;
    const GROWTH: f32 = 0.75;
    const MIN_LENGTH: f32 = 0.03;

    let cycle = phase(time, CYCLE);
    // Each cycle starts where the previous tail stopped so the arc never jumps.
    let completed = (time / CYCLE).floor();
    let base = (time / 2.0 + completed * f64::from(GROWTH)).rem_euclid(1.0) as f32;
    let head = GROWTH * ease_in_out((cycle * 2.0).min(1.0));
    let tail = GROWTH * ease_in_out((cycle * 2.0 - 1.0).max(0.0));

    (base + tail, base + head + MIN_LENGTH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inherited_defers_to_text_color() {
        let style = appearance(&Theme::default_dark(), Variant::Inherited);

        assert_eq!(style.indicator, None);
        assert_eq!(style.track, None);
    }

    #[test]
    fn semantic_uses_intent_indicator() {
        let theme = Theme::default_dark();
        let style = appearance(&theme, Variant::Semantic(Intent::Danger));

        assert_eq!(
            style.indicator,
            Some(theme.colors().danger.indicator.into())
        );
    }

    #[test]
    fn sweep_is_continuous_across_cycles() {
        let boundary = 1.4 * 3.0;
        let (before_start, before_end) = sweep_span(boundary - 1e-6);
        let (after_start, after_end) = sweep_span(boundary + 1e-6);
        let wrapped = |a: f32, b: f32| ((a - b + 0.5).rem_euclid(1.0) - 0.5).abs();

        assert!(wrapped(before_start, after_start) < 1e-3);
        assert!(wrapped(before_end, after_end) < 1e-3);
    }
}
