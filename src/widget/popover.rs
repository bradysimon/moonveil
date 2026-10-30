//! Popovers show floating content anchored to a base element.
//!
//! Pressing the base toggles the popover. A press outside the panel or `Esc` closes it.
//! Use [`Popover::is_open`] with [`Popover::on_open`] and [`Popover::on_close`] to control
//! the open state from the application instead.

use crate::{Element, Theme, spacing, widget::container};
use iced_core::{
    Border, Event, Layout, Length, Padding, Pixels, Rectangle, Shell, Size, Vector, Widget,
    keyboard::{self, key},
    layout::Limits,
    mouse, overlay, renderer, touch,
    widget::{Operation, Tree, tree},
};

/// The appearance of a popover panel.
pub use iced_widget::container::Style;

/// The default gap between a popover and its base.
pub const DEFAULT_GAP: Pixels = Pixels(spacing::XS);

/// The default padding around popover content.
pub const DEFAULT_PADDING: Padding = Padding::new(spacing::MD);

/// The default maximum width of a popover panel.
pub const DEFAULT_MAX_WIDTH: Pixels = Pixels(320.0);

/// The minimum distance kept between a popover and the window edges.
const EDGE_PADDING: f32 = 6.0;

/// Creates a popover that shows `content` when `base` is pressed.
pub fn popover<'a, Message, Renderer>(
    base: impl Into<Element<'a, Message, Renderer>>,
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Popover<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    Popover::new(base, content)
}

/// Where a popover is placed relative to its base.
///
/// The first word is the side of the base the popover appears on. `Start` and `End`
/// align the popover with the leading (left or top) or trailing (right or bottom) edge
/// of the base; otherwise the popover is centered on it.
///
/// A popover flips to the opposite side when its preferred side lacks room, and it is
/// always kept inside the window.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    /// Above the base, centered.
    Top,
    /// Above the base, aligned to its left edge.
    TopStart,
    /// Above the base, aligned to its right edge.
    TopEnd,
    /// Below the base, centered.
    Bottom,
    /// Below the base, aligned to its left edge.
    #[default]
    BottomStart,
    /// Below the base, aligned to its right edge.
    BottomEnd,
    /// Left of the base, centered.
    Left,
    /// Left of the base, aligned to its top edge.
    LeftStart,
    /// Left of the base, aligned to its bottom edge.
    LeftEnd,
    /// Right of the base, centered.
    Right,
    /// Right of the base, aligned to its top edge.
    RightStart,
    /// Right of the base, aligned to its bottom edge.
    RightEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Align {
    Start,
    Center,
    End,
}

impl Position {
    fn side(self) -> Side {
        match self {
            Self::Top | Self::TopStart | Self::TopEnd => Side::Top,
            Self::Bottom | Self::BottomStart | Self::BottomEnd => Side::Bottom,
            Self::Left | Self::LeftStart | Self::LeftEnd => Side::Left,
            Self::Right | Self::RightStart | Self::RightEnd => Side::Right,
        }
    }

    fn align(self) -> Align {
        match self {
            Self::Top | Self::Bottom | Self::Left | Self::Right => Align::Center,
            Self::TopStart | Self::BottomStart | Self::LeftStart | Self::RightStart => Align::Start,
            Self::TopEnd | Self::BottomEnd | Self::LeftEnd | Self::RightEnd => Align::End,
        }
    }
}

/// A floating panel of arbitrary content anchored to a base element.
pub struct Popover<'a, Message, Renderer = iced_widget::Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    base: Element<'a, Message, Renderer>,
    content: Element<'a, Message, Renderer>,
    position: Position,
    gap: f32,
    padding: Padding,
    width: Length,
    max_width: f32,
    disabled: bool,
    is_open: Option<bool>,
    on_open: Option<Message>,
    on_close: Option<Message>,
    class: <Theme as Catalog>::Class<'a>,
}

impl<'a, Message, Renderer> Popover<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    /// Creates a popover that shows `content` when `base` is pressed.
    pub fn new(
        base: impl Into<Element<'a, Message, Renderer>>,
        content: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        Self {
            base: base.into(),
            content: content.into(),
            position: Position::default(),
            gap: DEFAULT_GAP.0,
            padding: DEFAULT_PADDING,
            width: Length::Shrink,
            max_width: DEFAULT_MAX_WIDTH.0,
            disabled: false,
            is_open: None,
            on_open: None,
            on_close: None,
            class: <Theme as Catalog>::default(),
        }
    }

    /// Sets where the popover is placed relative to its base.
    #[must_use]
    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// Sets the distance between the base and the popover.
    #[must_use]
    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into().0.max(0.0);
        self
    }

    /// Sets the padding around the popover content.
    #[must_use]
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the popover panel.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the maximum width of the popover panel.
    #[must_use]
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0.max(0.0);
        self
    }

    /// Sets whether the popover is disabled. A disabled popover stays closed.
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the externally controlled open state.
    ///
    /// Pair with [`Popover::on_open`] and [`Popover::on_close`] to keep it in sync.
    #[must_use]
    pub fn is_open(mut self, is_open: bool) -> Self {
        self.is_open = Some(is_open);
        self
    }

    /// Sets the message published when the popover opens.
    #[must_use]
    pub fn on_open(mut self, message: Message) -> Self {
        self.on_open = Some(message);
        self
    }

    /// Sets the message published when the popover closes.
    #[must_use]
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Sets a built-in Moonveil popover variant.
    #[must_use]
    pub fn class(mut self, variant: Variant) -> Self {
        self.class = Class::Variant(variant);
        self
    }

    /// Sets a custom popover style function.
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self {
        self.class = Class::Custom(Box::new(style));
        self
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct State {
    is_open: bool,
    needs_relayout: bool,
    panel: Size,
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Popover<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.state.downcast_mut::<State>().needs_relayout = true;
        tree.diff_children(&mut [&mut self.base, &mut self.content]);
    }

    fn size(&self) -> Size<Length> {
        self.base.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) {
        let state = tree.state.downcast_mut::<State>();
        if let Some(is_open) = self.is_open {
            state.is_open = is_open;
        }
        if self.disabled {
            state.is_open = false;
        }
        state.needs_relayout = true;

        self.base
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        tree.size = tree.children[0].size;
        tree.children[0].translation = Vector::ZERO;
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.base.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            shell,
            viewport,
        );

        let state = tree.state.downcast_mut::<State>();
        if self.disabled || state.is_open {
            return;
        }

        // The base may capture the press (e.g. a button), so it is observed regardless.
        let pressed = match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                cursor.is_over(layout.bounds())
            }
            Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                layout.bounds().contains(*position)
            }
            _ => false,
        };

        if pressed {
            state.is_open = true;
            state.needs_relayout = true;
            if let Some(message) = &self.on_open {
                shell.publish(message.clone());
            }
            shell.invalidate_overlay();
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.base.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let interaction = self.base.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        );

        if !self.disabled
            && interaction <= mouse::Interaction::Idle
            && cursor.is_over(layout.bounds())
        {
            mouse::Interaction::Pointer
        } else {
            interaction
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.base.as_widget_mut().operate(
            &mut tree.children[0],
            layout,
            viewport,
            renderer,
            operation,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        let Tree {
            state, children, ..
        } = tree;
        let [base_tree, content_tree] = children.as_mut_slice() else {
            return Vec::new();
        };

        let mut overlays = self.base.as_widget_mut().overlay(
            base_tree,
            layout,
            renderer,
            viewport,
            translation,
            window,
        );

        let state = state.downcast_mut::<State>();
        if !state.is_open {
            return overlays;
        }

        if state.needs_relayout {
            let available = Size::new(
                (window.width - EDGE_PADDING * 2.0).clamp(0.0, self.max_width),
                (window.height - EDGE_PADDING * 2.0).max(0.0),
            );
            let limits = Limits::new(Size::ZERO, available).width(self.width);

            self.content.as_widget_mut().layout(
                content_tree,
                renderer,
                &limits.shrink(self.padding),
            );
            content_tree.translation = Vector::new(self.padding.left, self.padding.top);
            state.panel = limits.resolve(
                self.width,
                Length::Shrink,
                content_tree.size.expand(self.padding),
            );
            state.needs_relayout = false;
        }

        let panel = place(
            layout.bounds() + translation,
            state.panel,
            Rectangle::with_size(window),
            self.position,
            self.gap,
        );
        let content_layout =
            Layout::new(content_tree.size).move_to(panel.position() + content_tree.translation);

        overlays.push(overlay::Element::new(Box::new(PopoverOverlay {
            state,
            content: &mut self.content,
            tree: content_tree,
            panel,
            content_layout,
            on_close: self.on_close.as_ref(),
            class: &self.class,
            window,
        })));
        overlays
    }
}

impl<'a, Message, Renderer> From<Popover<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    fn from(popover: Popover<'a, Message, Renderer>) -> Self {
        Element::new(popover)
    }
}

struct PopoverOverlay<'a, 'b, Message, Renderer> {
    state: &'b mut State,
    content: &'b mut Element<'a, Message, Renderer>,
    tree: &'b mut Tree,
    panel: Rectangle,
    content_layout: Layout,
    on_close: Option<&'b Message>,
    class: &'b <Theme as Catalog>::Class<'a>,
    window: Size,
}

impl<Message: Clone, Renderer> PopoverOverlay<'_, '_, Message, Renderer> {
    fn close(&mut self, shell: &mut Shell<'_, Message>) {
        self.state.is_open = false;
        if let Some(message) = self.on_close {
            shell.publish(message.clone());
        }
        shell.capture_event();
        shell.invalidate_overlay();
        shell.request_redraw();
    }
}

impl<Message, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for PopoverOverlay<'_, '_, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    fn update(
        &mut self,
        event: &Event,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        self.content.as_widget_mut().update(
            self.tree,
            event,
            self.content_layout,
            cursor,
            renderer,
            shell,
            &Rectangle::with_size(self.window),
        );

        if shell.is_event_captured() {
            return;
        }

        let press = match event {
            Event::Mouse(mouse::Event::ButtonPressed(_)) => cursor.position(),
            Event::Touch(touch::Event::FingerPressed { position, .. }) => Some(*position),
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                if matches!(key.as_ref(), keyboard::Key::Named(key::Named::Escape)) =>
            {
                self.close(shell);
                return;
            }
            _ => None,
        };

        // Outside presses only dismiss, matching dropdown and context menus.
        if let Some(position) = press {
            if self.panel.contains(position) {
                shell.capture_event();
            } else {
                self.close(shell);
            }
        }
    }

    fn mouse_interaction(&self, cursor: mouse::Cursor, renderer: &Renderer) -> mouse::Interaction {
        if !cursor.is_over(self.panel) {
            return mouse::Interaction::None;
        }

        self.content
            .as_widget()
            .mouse_interaction(
                self.tree,
                self.content_layout,
                cursor,
                &Rectangle::with_size(self.window),
                renderer,
            )
            .max(mouse::Interaction::Idle)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        cursor: mouse::Cursor,
    ) {
        let viewport = Rectangle::with_size(self.window);
        let style = <Theme as Catalog>::style(theme, self.class);

        renderer.with_layer(viewport, |renderer| {
            iced_widget::container::draw_background(renderer, &style, self.panel);
            self.content.as_widget().draw(
                self.tree,
                renderer,
                theme,
                &renderer::Style {
                    text_color: style.text_color.unwrap_or(defaults.text_color),
                },
                self.content_layout,
                cursor,
                &viewport,
            );
        });
    }

    fn operate(&mut self, renderer: &Renderer, operation: &mut dyn Operation) {
        operation.container(None, self.panel, &self.panel);
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                self.tree,
                self.content_layout,
                &Rectangle::with_size(self.window),
                renderer,
                operation,
            );
        });
    }

    fn overlay<'c>(
        &'c mut self,
        renderer: &Renderer,
    ) -> Vec<overlay::Element<'c, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            self.tree,
            self.content_layout,
            renderer,
            &Rectangle::with_size(self.window),
            Vector::ZERO,
            self.window,
        )
    }
}

/// Places a panel of `size` next to `anchor`, flipping sides and clamping to stay within `viewport`.
fn place(
    anchor: Rectangle,
    size: Size,
    viewport: Rectangle,
    position: Position,
    gap: f32,
) -> Rectangle {
    let bounds = viewport.shrink(EDGE_PADDING);
    let side = flip(position.side(), anchor, size, bounds, gap);
    let align = |start: f32, length: f32, extent: f32| match position.align() {
        Align::Start => start,
        Align::Center => start + (length - extent) / 2.0,
        Align::End => start + length - extent,
    };

    let (x, y) = match side {
        Side::Top => (
            align(anchor.x, anchor.width, size.width),
            anchor.y - gap - size.height,
        ),
        Side::Bottom => (
            align(anchor.x, anchor.width, size.width),
            anchor.y + anchor.height + gap,
        ),
        Side::Left => (
            anchor.x - gap - size.width,
            align(anchor.y, anchor.height, size.height),
        ),
        Side::Right => (
            anchor.x + anchor.width + gap,
            align(anchor.y, anchor.height, size.height),
        ),
    };

    Rectangle {
        x: x.min(bounds.x + bounds.width - size.width).max(bounds.x),
        y: y.min(bounds.y + bounds.height - size.height).max(bounds.y),
        width: size.width,
        height: size.height,
    }
}

/// Returns the opposite side when `side` cannot fit the panel and the opposite side has more room.
fn flip(side: Side, anchor: Rectangle, size: Size, bounds: Rectangle, gap: f32) -> Side {
    let above = anchor.y - gap - bounds.y;
    let below = bounds.y + bounds.height - (anchor.y + anchor.height + gap);
    let before = anchor.x - gap - bounds.x;
    let after = bounds.x + bounds.width - (anchor.x + anchor.width + gap);

    let (space, opposite_space, needed, opposite) = match side {
        Side::Top => (above, below, size.height, Side::Bottom),
        Side::Bottom => (below, above, size.height, Side::Top),
        Side::Left => (before, after, size.width, Side::Right),
        Side::Right => (after, before, size.width, Side::Left),
    };

    if space < needed && opposite_space > space {
        opposite
    } else {
        side
    }
}

/// The theme catalog of a popover.
pub trait Catalog {
    /// The popover style class type.
    type Class<'a>;

    /// Returns the default popover class.
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a popover class into a style.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A custom popover style function using Moonveil's concrete theme.
pub type StyleFn<'a> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// A built-in Moonveil popover style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// A floating overlay surface with a standard edge and popover elevation.
    #[default]
    Standard,
    /// A floating overlay surface with a quieter edge.
    Subtle,
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

/// Returns the resolved style for a built-in popover variant.
pub fn appearance(theme: &Theme, variant: Variant) -> Style {
    let standard = container::appearance(theme, container::Variant::Popover);

    match variant {
        Variant::Standard => standard,
        Variant::Subtle => Style {
            border: Border {
                color: theme.colors().borders.subtle.into(),
                ..standard.border
            },
            ..standard
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: Rectangle = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    const ANCHOR: Rectangle = Rectangle {
        x: 300.0,
        y: 280.0,
        width: 100.0,
        height: 40.0,
    };
    const SIZE: Size = Size::new(200.0, 120.0);

    fn origin(anchor: Rectangle, position: Position) -> (f32, f32) {
        let bounds = place(anchor, SIZE, VIEWPORT, position, 4.0);
        assert_eq!(bounds.size(), SIZE);
        (bounds.x, bounds.y)
    }

    #[test]
    fn places_on_each_side_with_alignment() {
        assert_eq!(origin(ANCHOR, Position::Top), (250.0, 156.0));
        assert_eq!(origin(ANCHOR, Position::TopStart), (300.0, 156.0));
        assert_eq!(origin(ANCHOR, Position::TopEnd), (200.0, 156.0));
        assert_eq!(origin(ANCHOR, Position::Bottom), (250.0, 324.0));
        assert_eq!(origin(ANCHOR, Position::BottomStart), (300.0, 324.0));
        assert_eq!(origin(ANCHOR, Position::BottomEnd), (200.0, 324.0));
        assert_eq!(origin(ANCHOR, Position::Left), (96.0, 240.0));
        assert_eq!(origin(ANCHOR, Position::LeftStart), (96.0, 280.0));
        assert_eq!(origin(ANCHOR, Position::LeftEnd), (96.0, 200.0));
        assert_eq!(origin(ANCHOR, Position::Right), (404.0, 240.0));
        assert_eq!(origin(ANCHOR, Position::RightStart), (404.0, 280.0));
        assert_eq!(origin(ANCHOR, Position::RightEnd), (404.0, 200.0));
    }

    #[test]
    fn flips_when_preferred_side_lacks_room() {
        let near_bottom = Rectangle { y: 540.0, ..ANCHOR };
        assert_eq!(origin(near_bottom, Position::BottomStart), (300.0, 416.0));

        let near_top = Rectangle { y: 20.0, ..ANCHOR };
        assert_eq!(origin(near_top, Position::Top), (250.0, 64.0));

        let near_left = Rectangle { x: 20.0, ..ANCHOR };
        assert_eq!(origin(near_left, Position::LeftStart), (124.0, 280.0));
    }

    #[test]
    fn keeps_preferred_side_when_opposite_is_tighter() {
        let tall = Size::new(200.0, 400.0);
        let bounds = place(ANCHOR, tall, VIEWPORT, Position::BottomStart, 4.0);
        assert_eq!(bounds.y, VIEWPORT.height - EDGE_PADDING - tall.height);
    }

    #[test]
    fn clamps_cross_axis_inside_viewport() {
        let near_right = Rectangle { x: 740.0, ..ANCHOR };
        assert_eq!(
            origin(near_right, Position::BottomStart),
            (800.0 - EDGE_PADDING - SIZE.width, 324.0)
        );

        let near_left = Rectangle { x: 0.0, ..ANCHOR };
        assert_eq!(
            origin(near_left, Position::BottomEnd),
            (EDGE_PADDING, 324.0)
        );
    }
}
