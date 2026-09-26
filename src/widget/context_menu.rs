//! A context menu wrapper for showing action menus on right click.

mod defaults;
mod item;
mod menu;
mod shortcut;
#[cfg(test)]
mod tests;

use std::borrow::Cow;

use crate::{
    Element, Radii, Theme, icon,
    token::{Interaction, Surface},
};
use defaults::*;
use iced_core::{
    Background, Border, Color, Event, Layout, Length, Pixels, Point, Rectangle, Shadow, Shell,
    Size, Vector, Widget,
    keyboard::{self, key},
    layout, mouse, overlay,
    renderer::{self, Quad},
    svg,
    text::{self, Text},
    time::Instant,
    touch,
    widget::{Operation, Tree, tree},
    window,
};
pub use item::Item;
use menu::{Level, Metrics};
pub use shortcut::{Key, Shortcut};

/// Creates a context menu around a base element and menu items.
pub fn context_menu<'a, Message, Renderer>(
    base: impl Into<Element<'a, Message, Renderer>>,
    items: impl IntoIterator<Item = Item<'a, Message>>,
) -> ContextMenu<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: text::Renderer + svg::Renderer + 'a,
{
    ContextMenu::new(base, items)
}

/// Creates an enabled action item.
pub fn item<'a, Message>(title: impl Into<Cow<'a, str>>, message: Message) -> Item<'a, Message> {
    Item::Action {
        title: title.into(),
        on_press: Some(message),
        shortcut: None,
    }
}

/// Creates an action item that is enabled only when a message is present.
pub fn item_maybe<'a, Message>(
    title: impl Into<Cow<'a, str>>,
    message: Option<Message>,
) -> Item<'a, Message> {
    Item::Action {
        title: title.into(),
        on_press: message,
        shortcut: None,
    }
}

/// Creates a submenu item.
pub fn submenu<'a, Message>(
    title: impl Into<Cow<'a, str>>,
    items: impl IntoIterator<Item = Item<'a, Message>>,
) -> Item<'a, Message> {
    Item::Submenu {
        title: title.into(),
        items: items.into_iter().collect(),
        enabled: true,
    }
}

/// Creates a separator item.
pub fn separator<Message>() -> Item<'static, Message> {
    Item::Separator
}

/// A widget that opens a context menu when its base content is right-clicked.
pub struct ContextMenu<'a, Message, Renderer = iced_widget::Renderer>
where
    Message: Clone + 'a,
    Renderer: text::Renderer + svg::Renderer + 'a,
{
    base: Element<'a, Message, Renderer>,
    items: Vec<Item<'a, Message>>,
    width: f32,
    max_height: f32,
    class: <Theme as Catalog>::Class<'a>,
}

impl<'a, Message, Renderer> ContextMenu<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: text::Renderer + svg::Renderer + 'a,
{
    /// Creates a context menu around a base element and menu items.
    pub fn new(
        base: impl Into<Element<'a, Message, Renderer>>,
        items: impl IntoIterator<Item = Item<'a, Message>>,
    ) -> Self {
        Self {
            base: base.into(),
            items: items.into_iter().collect(),
            width: DEFAULT_WIDTH,
            max_height: MAX_HEIGHT,
            class: <Theme as Catalog>::default(),
        }
    }

    /// Sets the width of each menu level.
    #[must_use]
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = width.into().0.max(MIN_WIDTH);
        self
    }

    /// Sets the maximum height of each menu level before it scrolls.
    #[must_use]
    pub fn max_height(mut self, height: impl Into<Pixels>) -> Self {
        self.max_height = height.into().0.max(ITEM_HEIGHT * 2.0);
        self
    }

    /// Sets a custom menu style function.
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self {
        self.class = Class::Custom(Box::new(style));
        self
    }

    /// Sets the menu style variant.
    #[must_use]
    pub fn class(mut self, variant: Variant) -> Self {
        self.class = Class::Variant(variant);
        self
    }
}

#[derive(Debug, Clone, Default)]
struct State {
    /// The right-click position in the base widget's local coordinates.
    open_at: Option<Point>,
    active_path: Vec<usize>,
    scroll_offsets: Vec<f32>,
    scroll_drag: Option<ScrollDrag>,
    pending_safety_hover: Option<PendingSafetyHover>,
    /// Recent cursor positions in overlay coordinates, oldest first.
    pointer_trail: Vec<Point>,
}

#[derive(Debug, Clone, Copy)]
struct ScrollDrag {
    level: usize,
    cursor_offset: f32,
}

/// A hover target held back while the cursor travels toward an open submenu.
#[derive(Debug, Clone)]
struct PendingSafetyHover {
    path: Vec<usize>,
    started_at: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SafetyHoverAdvance {
    Idle,
    Waiting { until: Instant },
    Activated,
}

impl State {
    fn new() -> Self {
        Self::default()
    }

    fn open<Message>(&mut self, position: Point, items: &[Item<'_, Message>]) {
        self.open_at = Some(position);
        self.active_path = item::first_navigable_path(items).unwrap_or_default();
        self.scroll_offsets.clear();
        self.scroll_drag = None;
        self.pending_safety_hover = None;
        self.pointer_trail.clear();
    }

    fn close(&mut self) {
        *self = Self::default();
    }

    fn is_open(&self) -> bool {
        self.open_at.is_some()
    }

    fn move_next<Message>(&mut self, items: &[Item<'_, Message>]) {
        item::move_active(items, &mut self.active_path, 1);
        self.pending_safety_hover = None;
    }

    fn move_previous<Message>(&mut self, items: &[Item<'_, Message>]) {
        item::move_active(items, &mut self.active_path, -1);
        self.pending_safety_hover = None;
    }

    fn enter_submenu<Message>(&mut self, items: &[Item<'_, Message>]) {
        let Some(children) =
            item::item_at_path(items, &self.active_path).and_then(Item::submenu_items)
        else {
            return;
        };

        if let Some(index) = item::first_navigable_index(children) {
            self.active_path.push(index);
            self.pending_safety_hover = None;
        }
    }

    fn leave_submenu(&mut self) {
        if self.active_path.len() > 1 {
            self.active_path.pop();
            self.pending_safety_hover = None;
        }
    }

    fn set_hover_path(&mut self, path: Vec<usize>) {
        self.active_path = path;
        self.pending_safety_hover = None;
    }

    /// Records a cursor position and returns the oldest one in the trail, if any.
    fn record_pointer(&mut self, position: Point) -> Option<Point> {
        let oldest = self.pointer_trail.first().copied();
        if self.pointer_trail.len() >= POINTER_TRAIL_LEN {
            let _ = self.pointer_trail.remove(0);
        }
        self.pointer_trail.push(position);
        oldest
    }

    fn track_pending_safety_hover(&mut self, path: Vec<usize>) {
        if self
            .pending_safety_hover
            .as_ref()
            .is_some_and(|pending| pending.path == path)
        {
            return;
        }

        self.pending_safety_hover = Some(PendingSafetyHover {
            path,
            started_at: None,
        });
    }

    fn advance_pending_safety_hover(&mut self, now: Instant) -> SafetyHoverAdvance {
        let Some(pending) = &mut self.pending_safety_hover else {
            return SafetyHoverAdvance::Idle;
        };

        let started_at = *pending.started_at.get_or_insert(now);
        let until = started_at + SAFETY_HOVER_DELAY;

        if now < until {
            return SafetyHoverAdvance::Waiting { until };
        }

        if let Some(pending) = self.pending_safety_hover.take() {
            self.active_path = pending.path;
        }
        SafetyHoverAdvance::Activated
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for ContextMenu<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer + svg::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.base));
    }

    fn size(&self) -> Size<Length> {
        self.base.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        self.base
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        tree.size = tree.children[0].size;
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
        if tree.state.downcast_ref::<State>().is_open() {
            return;
        }

        self.base.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            shell,
            viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if let Some(position) = cursor.position_over(layout.bounds()) {
                    tree.state
                        .downcast_mut::<State>()
                        .open(position, &self.items);
                    shell.capture_event();
                    shell.invalidate_overlay();
                    shell.request_redraw();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                if let Some(message) = item::shortcut_message(&self.items, key, *modifiers) {
                    shell.publish(message.clone());
                    shell.capture_event();
                }
            }
            _ => {}
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
        self.base.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
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
        let state = tree.state.downcast_mut::<State>();

        if !state.is_open() {
            return self.base.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
                window,
            );
        }

        let overlay = ContextMenuOverlay {
            state,
            items: &self.items,
            width: self.width,
            max_height: self.max_height,
            translation,
            trigger: layout.bounds() + translation,
            class: &self.class,
            window,
        };
        let _ = menu::stack_mut(
            overlay.items,
            overlay.state,
            &overlay.metrics(Rectangle::with_size(window)),
        );

        vec![overlay::Element::new(Box::new(overlay))]
    }
}

impl<'a, Message, Renderer> From<ContextMenu<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: text::Renderer + svg::Renderer + 'a,
{
    fn from(menu: ContextMenu<'a, Message, Renderer>) -> Self {
        Element::new(menu)
    }
}

struct ContextMenuOverlay<'a, 'b, Message> {
    state: &'b mut State,
    items: &'b [Item<'a, Message>],
    width: f32,
    max_height: f32,
    translation: Vector,
    /// The base widget bounds in overlay coordinates.
    trigger: Rectangle,
    class: &'b <Theme as Catalog>::Class<'a>,
    window: Size,
}

impl<Message> ContextMenuOverlay<'_, '_, Message> {
    fn metrics(&self, viewport: Rectangle) -> Metrics {
        Metrics {
            width: self.width,
            max_height: self.max_height,
            viewport,
            translation: self.translation,
        }
    }

    fn viewport(&self) -> Rectangle {
        Rectangle::with_size(self.window)
    }
}

impl<Message: Clone> ContextMenuOverlay<'_, '_, Message> {
    fn handle_event(
        &mut self,
        event: &Event,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        let metrics = self.metrics(self.viewport());
        let menus = menu::stack_mut(self.items, self.state, &metrics);
        let position = cursor.land().position();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let Some(position) = position else {
                    return;
                };

                if let Some(drag) = self.state.scroll_drag
                    && let Some(menu) = menus.get(drag.level)
                {
                    menu::ensure_scroll_offsets_len(&mut self.state.scroll_offsets, drag.level + 1);
                    self.state.scroll_offsets[drag.level] = menu::scroll_offset_from_thumb_position(
                        menu,
                        position.y - drag.cursor_offset,
                    );
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }

                let from = self.state.record_pointer(position);

                match menu::path_at_position(&menus, position) {
                    Some(path) if path != self.state.active_path => {
                        if from
                            .is_some_and(|from| menu::is_aiming_at_submenu(&menus, from, position))
                        {
                            self.state.track_pending_safety_hover(path);
                        } else {
                            self.state.set_hover_path(path);
                        }
                        shell.request_redraw();
                    }
                    _ => self.state.pending_safety_hover = None,
                }

                if menu::contains(&menus, position) {
                    shell.capture_event();
                }
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                match self.state.advance_pending_safety_hover(*now) {
                    SafetyHoverAdvance::Idle => {}
                    SafetyHoverAdvance::Waiting { until } => shell.request_redraw_at(until),
                    SafetyHoverAdvance::Activated => shell.request_redraw(),
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if self.state.scroll_drag.take().is_some() {
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if shell.is_event_captured() {
                    return;
                }

                if let Some(position) = position
                    && let Some(level) = menu::scroll_level_at_position(&menus, position)
                {
                    let did_scroll =
                        menu::scroll_by(self.state, &menus, level, menu::scroll_delta_y(delta));
                    shell.capture_event();

                    if did_scroll {
                        shell.request_redraw();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let Some(position) = position else {
                    return;
                };

                if let Some((level, thumb)) = menu::scrollbar_thumb_at_position(&menus, position) {
                    self.state.scroll_drag = Some(ScrollDrag {
                        level,
                        cursor_offset: position.y - thumb.y,
                    });
                } else if let Some(path) = menu::path_at_position(&menus, position) {
                    if let Some(message) = item::active_message(self.items, &path) {
                        shell.publish(message.clone());
                        self.state.close();
                    } else {
                        self.state.set_hover_path(path);
                    }
                } else {
                    self.state.close();
                }

                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                let Some(position) = position else {
                    return;
                };

                if !menu::contains(&menus, position) {
                    if self.trigger.contains(position) {
                        self.state.open(position - self.translation, self.items);
                    } else {
                        self.state.close();
                    }
                    shell.request_redraw();
                }

                shell.capture_event();
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                if let Some(message) = item::shortcut_message(self.items, key, *modifiers) {
                    shell.publish(message.clone());
                    self.state.close();
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }

                match key.as_ref() {
                    keyboard::Key::Named(key::Named::Escape) => self.state.close(),
                    keyboard::Key::Named(key::Named::ArrowDown) => {
                        self.state.move_next(self.items);
                        menu::ensure_active_visible(self.items, self.state, &metrics);
                    }
                    keyboard::Key::Named(key::Named::ArrowUp) => {
                        self.state.move_previous(self.items);
                        menu::ensure_active_visible(self.items, self.state, &metrics);
                    }
                    keyboard::Key::Named(key::Named::ArrowRight) => {
                        self.state.enter_submenu(self.items);
                        menu::ensure_active_visible(self.items, self.state, &metrics);
                    }
                    keyboard::Key::Named(key::Named::ArrowLeft) => self.state.leave_submenu(),
                    keyboard::Key::Named(key::Named::Enter | key::Named::Space) => {
                        if let Some(message) =
                            item::active_message(self.items, &self.state.active_path)
                        {
                            shell.publish(message.clone());
                            self.state.close();
                        } else {
                            self.state.enter_submenu(self.items);
                            menu::ensure_active_visible(self.items, self.state, &metrics);
                        }
                    }
                    _ => return,
                }

                shell.capture_event();
                shell.request_redraw();
            }
            _ => {}
        }
    }
}

impl<Message, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for ContextMenuOverlay<'_, '_, Message>
where
    Message: Clone,
    Renderer: text::Renderer + svg::Renderer,
{
    fn update(
        &mut self,
        event: &Event,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        let was_open = self.state.is_open();
        self.handle_event(event, cursor, shell);

        if self.state.is_open() != was_open {
            shell.invalidate_overlay();
        }
    }

    fn mouse_interaction(&self, cursor: mouse::Cursor, _renderer: &Renderer) -> mouse::Interaction {
        if self.state.scroll_drag.is_some() {
            return mouse::Interaction::Grabbing;
        }

        let Some(position) = cursor.position() else {
            return mouse::Interaction::default();
        };

        let menus = menu::stack(self.items, self.state, &self.metrics(self.viewport()));

        if menu::scrollbar_thumb_at_position(&menus, position).is_some() {
            mouse::Interaction::Grab
        } else if menu::path_at_position(&menus, position)
            .is_some_and(|path| item::active_message(self.items, &path).is_some())
        {
            mouse::Interaction::Pointer
        } else if menu::contains(&menus, position) {
            mouse::Interaction::Idle
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _defaults: &renderer::Style,
        _cursor: mouse::Cursor,
    ) {
        let style = <Theme as Catalog>::style(theme, self.class);
        let viewport = self.viewport();
        let menus = menu::stack(self.items, self.state, &self.metrics(viewport));

        renderer.with_layer(viewport, |renderer| {
            for (level, menu) in menus.iter().enumerate() {
                draw_menu(
                    renderer,
                    &style,
                    menu,
                    self.state.active_path.get(level).copied(),
                );
            }
        });
    }
}

fn draw_menu<Message, Renderer>(
    renderer: &mut Renderer,
    style: &Style,
    menu: &Level<'_, Message>,
    active_index: Option<usize>,
) where
    Renderer: text::Renderer + svg::Renderer,
{
    renderer.with_layer(
        menu.bounds.expand(
            style.shadow.blur_radius + style.shadow.offset.x.abs().max(style.shadow.offset.y.abs()),
        ),
        |renderer| {
            renderer.fill_quad(
                Quad {
                    bounds: menu.bounds,
                    border: style.border,
                    shadow: style.shadow,
                    ..Quad::default()
                },
                style.background,
            );

            let content = menu::content_bounds(menu.bounds);
            renderer.with_layer(content, |renderer| {
                draw_items(renderer, style, menu, active_index, content);
            });

            if let Some(thumb) = menu::scrollbar_thumb_bounds(menu) {
                renderer.fill_quad(
                    Quad {
                        bounds: thumb,
                        border: Border::default()
                            .rounded(style.border.radius.top_left.min(SCROLLBAR_WIDTH / 2.0)),
                        ..Quad::default()
                    },
                    style.scrollbar_color,
                );
            }
        },
    );
}

fn draw_items<Message, Renderer>(
    renderer: &mut Renderer,
    style: &Style,
    menu: &Level<'_, Message>,
    active_index: Option<usize>,
    viewport: Rectangle,
) where
    Renderer: text::Renderer + svg::Renderer,
{
    let font = renderer.font();
    let line_height = text::LineHeight::Relative(1.2);

    for (index, item) in menu.items.iter().enumerate() {
        let bounds = menu::item_bounds(menu, index);
        let Some(visible) = bounds.intersection(&viewport) else {
            continue;
        };

        let Some(title) = item.title() else {
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle {
                        x: bounds.x + ITEM_HORIZONTAL_PADDING,
                        y: bounds.center_y(),
                        width: bounds.width - ITEM_HORIZONTAL_PADDING * 2.0,
                        height: 1.0,
                    },
                    ..Quad::default()
                },
                style.separator_color,
            );
            continue;
        };

        let enabled = item.is_enabled();
        if enabled && active_index == Some(index) {
            renderer.fill_quad(
                Quad {
                    bounds: visible,
                    border: Border::default()
                        .rounded(Radii::within(style.border.radius.top_left, MENU_PADDING)),
                    ..Quad::default()
                },
                style.hover_background,
            );
        }

        let (text_color, trailing_color) = if enabled {
            (style.text_color, style.shortcut_text_color)
        } else {
            (style.disabled_text_color, style.disabled_text_color)
        };
        let shortcut = item.shortcut_value();
        let is_submenu = matches!(item, Item::Submenu { .. });
        let trailing_width = if shortcut.is_some() {
            TRAILING_CONTENT_WIDTH
        } else if is_submenu {
            CHEVRON_SIZE
        } else {
            0.0
        };
        let right = bounds.x + bounds.width - ITEM_HORIZONTAL_PADDING;

        renderer.fill_text(
            Text {
                content: title.to_string(),
                bounds: Size::new(
                    bounds.width - ITEM_HORIZONTAL_PADDING * 2.0 - trailing_width,
                    bounds.height,
                ),
                size: Pixels(13.0),
                line_height,
                font,
                align_x: text::Alignment::Default,
                align_y: iced_core::alignment::Vertical::Center,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::None,
                ellipsis: text::Ellipsis::End,
                hint_factor: renderer.hint_factor(),
            },
            Point::new(bounds.x + ITEM_HORIZONTAL_PADDING, bounds.center_y()),
            text_color,
            viewport,
        );

        if let Some(shortcut) = shortcut {
            renderer.fill_text(
                Text {
                    content: shortcut.to_string(),
                    bounds: Size::new(TRAILING_CONTENT_WIDTH, bounds.height),
                    size: Pixels(12.0),
                    line_height,
                    font,
                    align_x: text::Alignment::Right,
                    align_y: iced_core::alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                    ellipsis: text::Ellipsis::End,
                    hint_factor: renderer.hint_factor(),
                },
                Point::new(right, bounds.center_y()),
                trailing_color,
                viewport,
            );
        } else if is_submenu {
            // Lucide's chevron glyph leaves ~5px of transparent space on the right of its box.
            renderer.draw_svg(
                svg::Svg::new(icon::chevron_right()).color(trailing_color),
                Rectangle {
                    x: right - CHEVRON_SIZE + 5.0,
                    y: bounds.center_y() - CHEVRON_SIZE / 2.0,
                    width: CHEVRON_SIZE,
                    height: CHEVRON_SIZE,
                },
                viewport,
            );
        }
    }
}

/// The appearance of a context menu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub background: Background,
    pub border: Border,
    pub text_color: Color,
    pub disabled_text_color: Color,
    /// The color of shortcut labels and submenu chevrons.
    pub shortcut_text_color: Color,
    pub hover_background: Background,
    pub separator_color: Color,
    pub scrollbar_color: Color,
    pub shadow: Shadow,
}

/// The theme catalog of a context menu.
pub trait Catalog {
    type Class<'a>;
    /// Returns the default context menu style class.
    fn default<'a>() -> Self::Class<'a>;
    /// Resolves a context menu appearance for a class.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a context menu.
pub type StyleFn<'a> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// A type of context menu style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    #[default]
    Standard,
    Subtle,
}

/// A built-in variant or downstream custom style function.
pub enum Class<'a> {
    Variant(Variant),
    Custom(StyleFn<'a>),
}

impl Default for Class<'_> {
    fn default() -> Self {
        Self::Variant(Variant::default())
    }
}

impl From<Variant> for Class<'_> {
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

/// Returns the resolved style for a built-in context menu variant.
pub fn appearance(theme: &Theme, variant: Variant) -> Style {
    let border = match variant {
        Variant::Standard => theme.colors().borders.standard,
        Variant::Subtle => theme.colors().borders.subtle,
    };

    Style {
        background: Background::Color(theme.surface(Surface::Overlay).into()),
        border: Border {
            color: border.into(),
            width: theme.appearance().border.hairline,
            radius: Radii::around(theme.appearance().radius.sm, MENU_PADDING).into(),
        },
        text_color: theme.colors().content.primary.into(),
        disabled_text_color: theme.colors().content.disabled.into(),
        shortcut_text_color: theme.colors().content.muted.into(),
        hover_background: Background::Color(
            theme
                .interaction_on(Surface::Overlay, Interaction::Selected)
                .into(),
        ),
        separator_color: theme.colors().borders.subtle.into(),
        scrollbar_color: theme.colors().content.muted.into(),
        shadow: theme.appearance().shadow.popover.into(),
    }
}
