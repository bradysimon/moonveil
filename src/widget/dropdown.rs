//! A custom dropdown wrapper for selectable action menus.

mod defaults;
mod item;
pub mod menu;
#[cfg(test)]
mod tests;

use std::borrow::Cow;

use crate::{
    Element, Radii, Theme,
    token::{Interaction, Surface},
};
use defaults::*;
use iced_core::{
    Background, Border, Color, Event, Layout, Length, Padding, Pixels, Point, Rectangle, Shadow,
    Shell, Size, Vector, Widget,
    keyboard::{self, key},
    layout::{self},
    mouse, overlay,
    renderer::{self, Quad},
    text::{self, Text},
    touch,
    widget::{Operation, Tree, tree},
    window,
};
pub use item::{Item, Leading};
use menu::Hit;

/// The default padding applied around dropdown content.
pub const PADDING: Padding = Padding {
    top: 4.0,
    bottom: 4.0,
    left: 8.0,
    right: 8.0,
};

/// Creates a dropdown around a base element and menu items.
pub fn dropdown<'a, Message, Renderer>(
    base: impl Into<Element<'a, Message, Renderer>>,
    items: impl IntoIterator<Item = Item<'a, Message, Renderer::Font>>,
) -> Dropdown<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: text::Renderer + 'a,
{
    Dropdown::new(base, items)
}

/// Creates an enabled action item.
pub fn item<'a, Message>(title: impl Into<Cow<'a, str>>, message: Message) -> Item<'a, Message> {
    Item::Action {
        title: title.into(),
        on_press: Some(message),
        leading: None,
    }
}
/// Creates an action item with an optional message.
pub fn item_maybe<'a, Message>(
    title: impl Into<Cow<'a, str>>,
    message: Option<Message>,
) -> Item<'a, Message> {
    Item::Action {
        title: title.into(),
        on_press: message,
        leading: None,
    }
}
/// Creates a non-action section heading.
pub fn section<'a, Message>(title: impl Into<Cow<'a, str>>) -> Item<'a, Message> {
    Item::Section {
        title: title.into(),
    }
}
/// Creates a separator item.
pub fn separator<Message>() -> Item<'static, Message> {
    Item::Separator
}

pub struct Dropdown<'a, Message, Renderer = iced_widget::Renderer>
where
    Message: Clone + 'a,
    Renderer: text::Renderer + 'a,
{
    base: Element<'a, Message, Renderer>,
    items: Vec<Item<'a, Message, Renderer::Font>>,
    padding: Padding,
    width: Option<f32>,
    max_height: f32,
    offset: f32,
    disabled: bool,
    is_open: Option<bool>,
    on_open: Option<Message>,
    on_close: Option<Message>,
    class: <Theme as Catalog>::Class<'a>,
    menu_class: <Theme as menu::Catalog>::Class<'a>,
    status: Option<Status>,
}

impl<'a, Message, Renderer> Dropdown<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: text::Renderer + 'a,
{
    /// Creates a dropdown around a base element and menu items.
    pub fn new(
        base: impl Into<Element<'a, Message, Renderer>>,
        items: impl IntoIterator<Item = Item<'a, Message, Renderer::Font>>,
    ) -> Self {
        Self {
            base: base.into(),
            items: items.into_iter().collect(),
            padding: PADDING,
            width: None,
            max_height: MAX_HEIGHT,
            offset: 4.0,
            disabled: false,
            is_open: None,
            on_open: None,
            on_close: None,
            class: <Theme as Catalog>::default(),
            menu_class: <Theme as menu::Catalog>::default(),
            status: None,
        }
    }

    /// Sets the dropdown menu width.
    #[must_use]
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = Some(width.into().0.max(MIN_WIDTH));
        self
    }

    #[must_use]
    /// Sets the dropdown menu maximum height.
    pub fn max_height(mut self, height: impl Into<Pixels>) -> Self {
        self.max_height = height.into().0.max(ITEM_HEIGHT * 2.0);
        self
    }

    /// Sets the distance between the trigger and menu.
    #[must_use]
    pub fn offset(mut self, offset: impl Into<Pixels>) -> Self {
        self.offset = offset.into().0.max(0.0);
        self
    }

    /// Sets the padding around the trigger content.
    #[must_use]
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets whether the dropdown is disabled.
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the externally controlled open state.
    #[must_use]
    pub fn is_open(mut self, is_open: bool) -> Self {
        self.is_open = Some(is_open);
        self
    }

    /// Sets the message published when the dropdown opens.
    #[must_use]
    pub fn on_open(mut self, message: Message) -> Self {
        self.on_open = Some(message);
        self
    }

    /// Sets the message published when the dropdown closes.
    #[must_use]
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Sets a custom trigger style function.
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self {
        self.class = Class::Custom(Box::new(style));
        self
    }

    /// Sets the trigger style variant.
    #[must_use]
    pub fn class(mut self, variant: Variant) -> Self {
        self.class = Class::Variant(variant);
        self
    }

    /// Sets a custom menu style function.
    #[must_use]
    pub fn menu_style(mut self, style: impl Fn(&Theme) -> menu::Style + 'a) -> Self
    where
        <Theme as menu::Catalog>::Class<'a>: From<menu::StyleFn<'a>>,
    {
        self.menu_class = (Box::new(style) as menu::StyleFn<'a>).into();
        self
    }

    /// Sets the menu style variant.
    #[must_use]
    pub fn menu_class(mut self, variant: menu::Variant) -> Self {
        self.menu_class = menu::Class::Variant(variant);
        self
    }
}

#[derive(Debug, Clone)]
struct State {
    is_open: bool,
    active_index: Option<usize>,
    scroll_offset: f32,
    scroll_drag: Option<ScrollDrag>,
}
#[derive(Debug, Clone, Copy)]
struct ScrollDrag {
    cursor_offset: f32,
}
impl State {
    fn new() -> Self {
        Self {
            is_open: false,
            active_index: None,
            scroll_offset: 0.0,
            scroll_drag: None,
        }
    }
    fn open<Message, Font>(&mut self, items: &[Item<'_, Message, Font>]) {
        self.is_open = true;
        self.active_index = item::first_navigable_index(items);
        self.scroll_offset = 0.0;
        self.scroll_drag = None;
    }
    fn close(&mut self) {
        self.is_open = false;
        self.active_index = None;
        self.scroll_offset = 0.0;
        self.scroll_drag = None;
    }
    fn move_next<Message, Font>(&mut self, items: &[Item<'_, Message, Font>]) {
        self.active_index = self
            .active_index
            .and_then(|index| item::next_navigable_index(items, index, 1))
            .or_else(|| item::first_navigable_index(items));
    }
    fn move_previous<Message, Font>(&mut self, items: &[Item<'_, Message, Font>]) {
        self.active_index = self
            .active_index
            .and_then(|index| item::next_navigable_index(items, index, -1))
            .or_else(|| item::last_navigable_index(items));
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Dropdown<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
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
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        if let Some(open) = self.is_open {
            let state = tree.state.downcast_mut::<State>();
            if open && !state.is_open {
                state.open(&self.items);
            } else if !open && state.is_open {
                state.close();
            }
        }
        layout::padded(limits, Length::Fit, Length::Fit, self.padding, |limits| {
            self.base
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        })
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let current_status = if self.disabled {
            Status::Disabled
        } else if state.is_open {
            Status::Opened
        } else if cursor.is_over(layout.bounds()) {
            Status::Hovered
        } else {
            Status::Active
        };
        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            self.status = Some(current_status);
        } else if self.status.is_some_and(|status| status != current_status) {
            shell.request_redraw();
        }
        if state.is_open {
            return;
        }
        if !self.disabled
            && matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    | Event::Touch(touch::Event::FingerPressed { .. })
            )
            && cursor.is_over(layout.bounds())
        {
            state.open(&self.items);
            if let Some(message) = &self.on_open {
                shell.publish(message.clone());
            }
            shell.capture_event();
            shell.request_redraw();
            return;
        }
        self.base.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            shell,
            viewport,
        );
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let status = if self.disabled {
            Status::Disabled
        } else if state.is_open {
            Status::Opened
        } else if cursor.is_over(layout.bounds()) {
            Status::Hovered
        } else {
            Status::Active
        };
        let appearance = <Theme as Catalog>::style(theme, &self.class, status);
        renderer.fill_quad(
            Quad {
                bounds: layout.bounds(),
                border: appearance.border,
                shadow: appearance.shadow,
                snap: appearance.snap,
            },
            appearance.background,
        );
        self.base.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: appearance.text_color,
            },
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if !self.disabled && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            self.base.as_widget().mouse_interaction(
                &tree.children[0],
                layout.children().next().unwrap(),
                cursor,
                viewport,
                renderer,
            )
        }
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.base.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State>();
        let base_overlay = if state.is_open {
            None
        } else {
            self.base.as_widget_mut().overlay(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                viewport,
                translation,
            )
        };
        let menu_overlay = state.is_open.then(|| {
            overlay::Element::new(Box::new(DropdownOverlay {
                state,
                items: &self.items,
                width: self.width,
                max_height: self.max_height,
                offset: self.offset,
                trigger: layout.bounds() + translation,
                on_close: self.on_close.as_ref(),
                class: &self.menu_class,
            }))
        });
        if base_overlay.is_some() || menu_overlay.is_some() {
            Some(
                overlay::Group::with_children(
                    base_overlay.into_iter().chain(menu_overlay).collect(),
                )
                .overlay(),
            )
        } else {
            None
        }
    }
}

impl<'a, Message, Renderer> From<Dropdown<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(value: Dropdown<'a, Message, Renderer>) -> Self {
        Element::new(value)
    }
}

struct DropdownOverlay<'a, 'b, Message, Font> {
    state: &'b mut State,
    items: &'b [Item<'a, Message, Font>],
    width: Option<f32>,
    max_height: f32,
    offset: f32,
    trigger: Rectangle,
    on_close: Option<&'b Message>,
    class: &'b <Theme as menu::Catalog>::Class<'a>,
}

impl<Message, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for DropdownOverlay<'_, '_, Message, Renderer::Font>
where
    Message: Clone,
    Renderer: text::Renderer,
{
    fn layout(&mut self, _renderer: &Renderer, bounds: Size) -> layout::Node {
        let menu = menu::layout(
            self.items,
            self.state.scroll_offset,
            self.trigger,
            self.width,
            self.max_height,
            self.offset,
            Rectangle::with_size(bounds),
        );
        menu::clamp_scroll_offset(&mut self.state.scroll_offset, &menu);
        layout::Node::new(bounds)
    }
    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        let menu = menu::layout(
            self.items,
            self.state.scroll_offset,
            self.trigger,
            self.width,
            self.max_height,
            self.offset,
            layout.bounds(),
        );
        let position = cursor.land().position();
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(position) = position
                    && let Some(drag) = self.state.scroll_drag
                {
                    self.state.scroll_offset = menu::scroll_offset_from_thumb_position(
                        &menu,
                        position.y - drag.cursor_offset,
                    );
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
                if let Some(position) = position {
                    let hit = menu::hit_test(self.items, &menu, position);
                    let next = match hit {
                        Hit::Row(index) if self.items[index].is_enabled() => Some(index),
                        Hit::Row(_) | Hit::Chrome => None,
                        Hit::Outside => self.state.active_index,
                    };
                    if next != self.state.active_index {
                        self.state.active_index = next;
                        shell.request_redraw();
                    }
                    if hit != Hit::Outside {
                        shell.capture_event();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if self.state.scroll_drag.take().is_some() {
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if let Some(position) = position
                    && menu::hit_test(self.items, &menu, position) != Hit::Outside
                {
                    let changed = menu::scroll_menu_by(&mut self.state.scroll_offset, &menu, delta);
                    shell.capture_event();
                    if changed {
                        shell.request_redraw();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(position) = position {
                    if let Some(thumb) = menu::scrollbar_thumb_at_position(&menu, position) {
                        self.state.scroll_drag = Some(ScrollDrag {
                            cursor_offset: position.y - thumb.y,
                        });
                        shell.capture_event();
                        shell.request_redraw();
                        return;
                    }
                    match menu::hit_test(self.items, &menu, position) {
                        Hit::Row(index) => {
                            self.state.active_index =
                                self.items[index].is_enabled().then_some(index);
                            if let Some(message) = self.items[index].message() {
                                shell.publish(message.clone());
                                close(self.state, self.on_close, shell);
                            }
                            shell.capture_event();
                            shell.request_redraw();
                        }
                        Hit::Chrome => {
                            self.state.active_index = None;
                            shell.capture_event();
                            shell.request_redraw();
                        }
                        Hit::Outside => {
                            close(self.state, self.on_close, shell);
                            shell.capture_event();
                            shell.request_redraw();
                        }
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => match key.as_ref() {
                keyboard::Key::Named(key::Named::Escape) => {
                    close(self.state, self.on_close, shell);
                    shell.capture_event();
                    shell.request_redraw();
                }
                keyboard::Key::Named(key::Named::ArrowDown) => {
                    self.state.move_next(self.items);
                    ensure_active_visible(self.items, self.state, &menu);
                    shell.capture_event();
                    shell.request_redraw();
                }
                keyboard::Key::Named(key::Named::ArrowUp) => {
                    self.state.move_previous(self.items);
                    ensure_active_visible(self.items, self.state, &menu);
                    shell.capture_event();
                    shell.request_redraw();
                }
                keyboard::Key::Named(key::Named::Home) => {
                    self.state.active_index = item::first_navigable_index(self.items);
                    ensure_active_visible(self.items, self.state, &menu);
                    shell.capture_event();
                    shell.request_redraw();
                }
                keyboard::Key::Named(key::Named::End) => {
                    self.state.active_index = item::last_navigable_index(self.items);
                    ensure_active_visible(self.items, self.state, &menu);
                    shell.capture_event();
                    shell.request_redraw();
                }
                keyboard::Key::Named(key::Named::Enter)
                | keyboard::Key::Named(key::Named::Space) => {
                    if let Some(index) = self.state.active_index
                        && let Some(message) = self.items[index].message()
                    {
                        shell.publish(message.clone());
                        close(self.state, self.on_close, shell);
                        shell.request_redraw();
                    }
                    shell.capture_event();
                }
                _ => {}
            },
            _ => {}
        }
    }
    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let menu = menu::layout(
            self.items,
            self.state.scroll_offset,
            self.trigger,
            self.width,
            self.max_height,
            self.offset,
            layout.bounds(),
        );
        if self.state.scroll_drag.is_some() {
            return mouse::Interaction::Grabbing;
        }
        let Some(position) = cursor.position() else {
            return mouse::Interaction::default();
        };
        if menu::scrollbar_thumb_at_position(&menu, position).is_some() {
            return mouse::Interaction::Grab;
        }
        match menu::hit_test(self.items, &menu, position) {
            Hit::Row(index) if self.items[index].message().is_some() => mouse::Interaction::Pointer,
            Hit::Row(_) | Hit::Chrome => mouse::Interaction::Idle,
            Hit::Outside => mouse::Interaction::default(),
        }
    }
    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let style = <Theme as menu::Catalog>::style(theme, self.class);
        let menu = menu::layout(
            self.items,
            self.state.scroll_offset,
            self.trigger,
            self.width,
            self.max_height,
            self.offset,
            layout.bounds(),
        );
        draw_menu(
            renderer,
            &style,
            self.items,
            &menu,
            self.state.active_index,
            cursor,
        );
    }
}

fn close<Message: Clone>(
    state: &mut State,
    on_close: Option<&Message>,
    shell: &mut Shell<'_, Message>,
) {
    state.close();
    if let Some(message) = on_close {
        shell.publish(message.clone());
    }
}
fn ensure_active_visible<Message, Font>(
    items: &[Item<'_, Message, Font>],
    state: &mut State,
    menu: &menu::Layout,
) {
    let Some(index) = state.active_index else {
        return;
    };
    let row = menu::item_bounds(items, index, menu);
    let top = menu.bounds.y + MENU_PADDING;
    let bottom = menu.bounds.y + menu.bounds.height - MENU_PADDING;
    if row.y < top {
        state.scroll_offset -= top - row.y;
    } else if row.y + row.height > bottom {
        state.scroll_offset += row.y + row.height - bottom;
    }
    menu::clamp_scroll_offset(&mut state.scroll_offset, menu);
}

fn draw_menu<Message, Renderer: text::Renderer>(
    renderer: &mut Renderer,
    style: &menu::Style,
    items: &[Item<'_, Message, Renderer::Font>],
    menu: &menu::Layout,
    active_index: Option<usize>,
    cursor: mouse::Cursor,
) {
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
            let content = menu::content_bounds(menu);
            renderer.with_layer(content, |renderer| {
                draw_items(renderer, style, items, menu, active_index, content)
            });
            if menu.content_height > menu.bounds.height {
                draw_scroll_indicator(renderer, style, menu, cursor);
            }
        },
    );
}

fn draw_items<Message, Renderer: text::Renderer>(
    renderer: &mut Renderer,
    style: &menu::Style,
    items: &[Item<'_, Message, Renderer::Font>],
    menu: &menu::Layout,
    active_index: Option<usize>,
    viewport: Rectangle,
) {
    let font = renderer.default_font();
    for (index, item) in items.iter().enumerate() {
        let bounds = menu::item_bounds(items, index, menu);
        let Some(visible) = bounds.intersection(&viewport) else {
            continue;
        };
        match item {
            Item::Separator => renderer.fill_quad(
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
            ),
            Item::Section { title } => renderer.fill_text(
                Text {
                    content: title.to_string(),
                    bounds: Size::new(bounds.width - ITEM_HORIZONTAL_PADDING * 2.0, bounds.height),
                    size: Pixels(11.0),
                    line_height: text::LineHeight::Relative(1.2),
                    font,
                    align_x: text::Alignment::Default,
                    align_y: iced_core::alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                    ellipsis: text::Ellipsis::End,
                    hint_factor: renderer.hint_factor(),
                },
                Point::new(bounds.x + ITEM_HORIZONTAL_PADDING, bounds.center_y()),
                style.section_text_color,
                viewport,
            ),
            Item::Action {
                title,
                on_press,
                leading,
            } => {
                let enabled = on_press.is_some();
                if active_index == Some(index) && enabled {
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
                let leading_width = if menu.has_leading {
                    LEADING_SIZE + LEADING_GAP
                } else {
                    0.0
                };
                if let Some(leading) = leading {
                    draw_leading(
                        renderer,
                        leading,
                        Rectangle {
                            x: bounds.x + ITEM_HORIZONTAL_PADDING,
                            y: bounds.center_y() - LEADING_SIZE / 2.0,
                            width: LEADING_SIZE,
                            height: LEADING_SIZE,
                        },
                        if enabled {
                            style.leading_color
                        } else {
                            style.disabled_text_color
                        },
                        viewport,
                    );
                }
                renderer.fill_text(
                    Text {
                        content: title.to_string(),
                        bounds: Size::new(
                            bounds.width - ITEM_HORIZONTAL_PADDING * 2.0 - leading_width,
                            bounds.height,
                        ),
                        size: Pixels(13.0),
                        line_height: text::LineHeight::Relative(1.2),
                        font,
                        align_x: text::Alignment::Default,
                        align_y: iced_core::alignment::Vertical::Center,
                        shaping: text::Shaping::Basic,
                        wrapping: text::Wrapping::None,
                        ellipsis: text::Ellipsis::End,
                        hint_factor: renderer.hint_factor(),
                    },
                    Point::new(
                        bounds.x + ITEM_HORIZONTAL_PADDING + leading_width,
                        bounds.center_y(),
                    ),
                    if enabled {
                        style.text_color
                    } else {
                        style.disabled_text_color
                    },
                    viewport,
                );
            }
        }
    }
}

fn draw_leading<Renderer: text::Renderer>(
    renderer: &mut Renderer,
    leading: &Leading<Renderer::Font>,
    bounds: Rectangle,
    color: Color,
    viewport: Rectangle,
) {
    let mut buffer = [0; 4];
    let content = leading.code_point.encode_utf8(&mut buffer);
    renderer.fill_text(
        Text {
            content: content.to_string(),
            bounds: bounds.size(),
            size: leading.size.unwrap_or(Pixels(bounds.height)),
            line_height: leading.line_height,
            font: leading.font,
            align_x: text::Alignment::Center,
            align_y: iced_core::alignment::Vertical::Center,
            shaping: leading.shaping,
            wrapping: text::Wrapping::None,
            ellipsis: text::Ellipsis::None,
            hint_factor: renderer.hint_factor(),
        },
        bounds.center(),
        color,
        viewport,
    );
}

fn draw_scroll_indicator<Renderer: renderer::Renderer>(
    renderer: &mut Renderer,
    style: &menu::Style,
    menu: &menu::Layout,
    _cursor: mouse::Cursor,
) {
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub background: Background,
    pub border: Border,
    pub text_color: Color,
    pub shadow: Shadow,
    pub snap: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Active,
    Hovered,
    Opened,
    Disabled,
}

pub trait Catalog {
    type Class<'a>;
    /// Returns the default dropdown style class.
    fn default<'a>() -> Self::Class<'a>;
    /// Resolves a dropdown appearance for a class and status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}
pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    #[default]
    Standard,
    Subtle,
}
pub enum Class<'a> {
    Variant(Variant),
    Custom(StyleFn<'a>),
}
impl Default for Class<'_> {
    fn default() -> Self {
        Self::Variant(Variant::default())
    }
}
impl<'a> From<Variant> for Class<'a> {
    fn from(value: Variant) -> Self {
        Self::Variant(value)
    }
}
impl<'a> From<StyleFn<'a>> for Class<'a> {
    fn from(value: StyleFn<'a>) -> Self {
        Self::Custom(value)
    }
}
impl Catalog for Theme {
    type Class<'a> = Class<'a>;
    fn default<'a>() -> Self::Class<'a> {
        Class::default()
    }
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        match class {
            Class::Variant(variant) => appearance(self, status, *variant),
            Class::Custom(style) => style(self, status),
        }
    }
}

/// Returns the appearance for a dropdown variant and status.
pub fn appearance(theme: &Theme, status: Status, variant: Variant) -> Style {
    let (background, border) = match variant {
        Variant::Standard => (
            Some(match status {
                Status::Active | Status::Disabled => theme.surface(Surface::Raised),
                Status::Hovered => theme.interaction_on(Surface::Raised, Interaction::Hover),
                Status::Opened => theme.interaction_on(Surface::Raised, Interaction::Selected),
            }),
            theme.colors().borders.standard,
        ),
        Variant::Subtle => (
            match status {
                Status::Active | Status::Disabled => None,
                Status::Hovered => Some(theme.colors().interaction.hover),
                Status::Opened => Some(theme.colors().interaction.selected),
            },
            theme.colors().borders.subtle,
        ),
    };
    Style {
        background: background.map_or(Background::Color(iced_core::Color::TRANSPARENT), |color| {
            Background::Color(color.into())
        }),
        border: Border {
            color: border.into(),
            width: if variant == Variant::Subtle {
                0.0
            } else {
                theme.appearance().border.hairline
            },
            radius: theme.appearance().radius.sm.into(),
        },
        text_color: if status == Status::Disabled {
            theme.colors().content.disabled.into()
        } else {
            theme.colors().content.primary.into()
        },
        shadow: Shadow::default(),
        snap: true,
    }
}
