//! A composable dialog wrapper that can show modal content over a base element.
//!
//! The dialog lifecycle is externally controlled with [`State`] and lifecycle
//! events are emitted as [`Message`] so parent state can keep content mounted
//! until close animations are fully complete.

use crate::{
    Element, Theme, icon,
    widget::{button, column, container, row, svg, text},
};
use iced_core::{
    Alignment, Animation, Color, Event, Length, Rectangle, Size, Vector,
    alignment::Horizontal::Right,
    animation, keyboard,
    layout::{Layout, Limits},
    mouse, overlay,
    renderer::{self, Quad},
    time::{Duration, Instant},
    touch,
    widget::{Operation, Tree, tree},
    window,
};
use iced_widget::space;

/// The default width used by a dialog panel.
pub const DEFAULT_WIDTH: Length = Length::Fixed(400.0);

/// Dialog lifecycle message emitted by the dialog widget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Message {
    /// The dialog has finished opening.
    Opened,
    /// The user requested the dialog to close.
    Close,
    /// The dialog has finished closing.
    /// Primarily used for animated dialogs to signal the end of the closing animation.
    Closed,
}

/// Actions that the dialog informs the main app about.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    /// The dialog has been closed and the parent app should clean up any necessary state.
    Closed,
}

/// The visual status of the dialog.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Status {
    /// The dialog is fully closed.
    #[default]
    Closed,
    /// The dialog is in the process of opening and animating in.
    Opening,
    /// The dialog is open and visible.
    Open,
    /// The dialog is in the process of closing and animating out.
    Closing,
}

/// External dialog state managed by the parent app.
#[derive(Debug, Clone, Copy)]
pub struct State {
    /// The current visual status of the dialog.
    /// Animated dialogs will use [`Status::Opening`] and [`Status::Closing`] states.
    status: Status,
    /// Whether the dialog should animate when opening or closing.
    is_animated: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            status: Status::Closed,
            is_animated: true,
        }
    }
}

impl State {
    /// Sets whether the dialog should animate when opening or closing.
    pub fn animated(mut self, animated: bool) -> Self {
        self.is_animated = animated;
        self
    }

    /// Sets the dialog state to open.
    pub fn open(&mut self) {
        self.status = if self.is_animated {
            Status::Opening
        } else {
            Status::Open
        };
    }

    /// Closes the dialog, either immediately closing if not animated or starting the closing animation.
    pub fn close(&mut self) {
        if self.is_animated {
            self.status = Status::Closing;
        } else {
            self.status = Status::Closed;
        }
    }

    /// Applies a dialog lifecycle message to this state.
    #[must_use]
    pub fn update(&mut self, message: Message) -> Option<Action> {
        match message {
            Message::Opened => {
                self.status = Status::Open;
                None
            }
            Message::Close => {
                if self.is_animated {
                    if self.status != Status::Closing {
                        self.status = Status::Closing;
                    }
                    None
                } else {
                    self.status = Status::Closed;
                    Some(Action::Closed)
                }
            }
            Message::Closed => {
                self.status = Status::Closed;
                Some(Action::Closed)
            }
        }
    }

    /// Returns the current status.
    pub fn status(&self) -> Status {
        self.status
    }

    /// Returns whether this dialog state is configured to animate.
    pub fn is_animated(&self) -> bool {
        self.is_animated
    }

    /// Returns true when the dialog target state is open.
    pub fn is_open(&self) -> bool {
        matches!(self.status, Status::Opening | Status::Open)
    }

    /// Returns true while dialog content should be rendered.
    pub fn is_visible(&self) -> bool {
        self.status != Status::Closed
    }
}

/// The resolved visual style of a dialog.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The color drawn behind the dialog panel.
    pub scrim: Color,
}

/// A custom dialog style function using Moonveil's concrete theme.
pub type StyleFn<'a> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// A built-in Moonveil dialog style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// The standard modal dialog treatment.
    #[default]
    Standard,
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

/// The theme catalog used to style a dialog.
pub trait Catalog {
    /// The dialog style class type.
    type Class<'a>;

    /// Returns the default dialog class.
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a dialog class into a style.
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

/// Returns the resolved style for a built-in dialog variant.
pub fn appearance(theme: &Theme, variant: Variant) -> Style {
    match variant {
        Variant::Standard => Style {
            scrim: theme.colors().surfaces.scrim.into(),
        },
    }
}

/// Creates a new [`Dialog`] wrapping the given `base` element.
///
/// The external [`State`] controls whether the dialog target is open or closing.
pub fn dialog<'a, Message, Renderer>(
    base: impl Into<Element<'a, Message, Renderer>>,
    state: &'a State,
    config: Option<Config<'a, Message, Renderer>>,
) -> Dialog<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    Dialog::new(base, state, config)
}

/// Describes what to show inside a [`Dialog`] panel.
///
/// Bundles the body content, optional title, and footer action buttons into a
/// single value that can be passed as `Option<Config>` to [`dialog()`].
pub struct Config<'a, Message, Renderer = iced_widget::Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    /// Optional title shown in the dialog header.
    title: Option<String>,
    /// Optional close button label.
    close_label: Option<String>,
    /// The width of the dialog panel.
    width: Length,
    /// Body content shown inside the dialog panel.
    content: Element<'a, Message, Renderer>,
    /// Footer action widgets rendered at the bottom of the panel.
    actions: Vec<Element<'a, Message, Renderer>>,
}

impl<'a, Message, Renderer> Config<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    /// Creates a new [`Config`] with the given body content.
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Self {
            title: None,
            close_label: None,
            width: DEFAULT_WIDTH,
            content: content.into(),
            actions: Vec::new(),
        }
    }

    /// Sets an optional dialog title shown in the header.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the text shown by the close button.
    #[must_use]
    pub fn close_label(mut self, label: impl Into<String>) -> Self {
        self.close_label = Some(label.into());
        self
    }

    /// Sets the width of the dialog panel.
    ///
    /// Some designs like Material recommend widths between 280-560px.
    /// See: [`DEFAULT_WIDTH`]
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Adds an action element to the dialog footer.
    #[must_use]
    pub fn push_action(mut self, action: impl Into<Element<'a, Message, Renderer>>) -> Self {
        self.actions.push(action.into());
        self
    }

    /// Adds multiple action elements to the dialog footer.
    #[must_use]
    pub fn actions(
        mut self,
        actions: impl IntoIterator<Item = Element<'a, Message, Renderer>>,
    ) -> Self {
        self.actions.extend(actions);
        self
    }
}

/// A composable dialog wrapper that can show modal content over a base element.
///
/// Use [`dialog()`] to create a `Dialog`, passing `Some(config)` to show a
/// dialog or `None` to hide it. The dialog will animate open/closed
/// automatically when the config transitions between `Some` and `None`.
pub struct Dialog<'a, Message, Renderer = iced_widget::Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    base: Element<'a, Message, Renderer>,
    content: Element<'a, Message, Renderer>,
    open: bool,
    on_update: Option<Box<dyn Fn(self::Message) -> Message + 'a>>,
    title: Option<String>,
    close_label: Option<String>,
    width: Length,
    backdrop_close: bool,
    esc_close: bool,
    animate: bool,
    actions: Vec<Element<'a, Message, Renderer>>,
    class: <Theme as Catalog>::Class<'a>,
}

impl<'a, Message, Renderer> Dialog<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    /// Creates a new [`Dialog`] with the given base element and optional config.
    pub fn new(
        base: impl Into<Element<'a, Message, Renderer>>,
        state: &State,
        config: Option<Config<'a, Message, Renderer>>,
    ) -> Self {
        let (content, title, close_label, width, actions, open) = match config {
            Some(config) => (
                config.content,
                config.title,
                config.close_label,
                config.width,
                config.actions,
                state.is_open(),
            ),
            None => (space().into(), None, None, DEFAULT_WIDTH, Vec::new(), false),
        };

        Self {
            base: base.into(),
            content,
            open,
            on_update: None,
            title,
            close_label,
            width,
            backdrop_close: true,
            esc_close: true,
            animate: state.is_animated(),
            actions,
            class: <Theme as Catalog>::default(),
        }
    }

    /// Sets the app message mapper for dialog lifecycle events.
    #[must_use]
    pub fn on_update(mut self, mapper: impl Fn(self::Message) -> Message + 'a) -> Self {
        self.on_update = Some(Box::new(mapper));
        self
    }

    /// Sets whether clicking the backdrop will close the dialog.
    #[must_use]
    pub fn backdrop_close(mut self, enabled: bool) -> Self {
        self.backdrop_close = enabled;
        self
    }

    /// Sets whether pressing `Esc` will close the dialog.
    #[must_use]
    pub fn esc_close(mut self, enabled: bool) -> Self {
        self.esc_close = enabled;
        self
    }

    /// Sets a built-in Moonveil dialog variant.
    #[must_use]
    pub fn class(mut self, variant: Variant) -> Self {
        self.class = Class::Variant(variant);
        self
    }

    /// Sets a custom dialog style function.
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self {
        self.class = Class::Custom(Box::new(style));
        self
    }

    fn build_overlay_parts(
        content: Element<'a, Message, Renderer>,
        title_text: Option<String>,
        close_label: Option<String>,
        width: Length,
        actions: Vec<Element<'a, Message, Renderer>>,
        close_message: Option<Message>,
        backdrop_close: bool,
    ) -> (
        Element<'a, Message, Renderer>,
        Element<'a, Message, Renderer>,
    )
    where
        Renderer: iced_core::svg::Renderer + iced_core::text::Renderer,
    {
        let close_content: Element<'a, Message, Renderer> = if let Some(label) = close_label {
            row([
                svg(icon::close())
                    .width(14)
                    .height(14)
                    .class(svg::Variant::Primary)
                    .into(),
                text(label).size(14).into(),
            ])
            .spacing(6)
            .align_y(Alignment::Center)
            .into()
        } else {
            svg(icon::close())
                .width(16)
                .height(16)
                .class(svg::Variant::Primary)
                .into()
        };

        let close_button: Element<'a, Message, Renderer> = {
            let control = button(close_content)
                .padding(6)
                .class(button::Variant::Ghost);

            if let Some(message) = close_message.clone() {
                control.on_press(message).into()
            } else {
                control.into()
            }
        };

        let title: Element<'a, Message, Renderer> = title_text
            .map(|title| text(title).size(18).into())
            .unwrap_or_else(|| space().into());

        let header =
            row(vec![title, space::horizontal().into(), close_button]).align_y(Alignment::Center);
        let mut body = column(vec![header.into(), content]).spacing(8);

        if !actions.is_empty() {
            body = body.push(
                container(row(actions).spacing(8).align_y(Alignment::Center))
                    .width(Length::Fill)
                    .align_x(Right),
            );
        }

        let panel: Element<'a, Message, Renderer> = container(body)
            .padding(16)
            .width(width)
            .class(container::Variant::Modal)
            .into();

        let backdrop_target = if backdrop_close {
            if let Some(message) = close_message {
                iced_widget::mouse_area(space().width(Length::Fill).height(Length::Fill))
                    .on_press(message)
                    .into()
            } else {
                space().width(Length::Fill).height(Length::Fill).into()
            }
        } else {
            space().width(Length::Fill).height(Length::Fill).into()
        };

        (backdrop_target, panel)
    }
}

impl<'a, Message, Renderer> From<Dialog<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + iced_core::svg::Renderer + iced_core::text::Renderer + 'a,
{
    fn from(dialog: Dialog<'a, Message, Renderer>) -> Self {
        let Dialog {
            base,
            content,
            open,
            on_update,
            title,
            close_label,
            width,
            backdrop_close,
            esc_close,
            animate,
            actions,
            class,
        } = dialog;

        let close_intent_message = on_update.as_ref().map(|map| map(self::Message::Close));
        let on_opened = on_update.as_ref().map(|map| map(self::Message::Opened));
        let on_closed = on_update.as_ref().map(|map| map(self::Message::Closed));

        let (backdrop_target, panel) = Dialog::build_overlay_parts(
            content,
            title,
            close_label,
            width,
            actions,
            close_intent_message.clone(),
            backdrop_close,
        );

        Element::new(ResolvedDialog {
            base,
            backdrop_target,
            panel,
            open,
            on_opened,
            on_close_intent: close_intent_message,
            on_closed,
            esc_close,
            animate,
            class,
        })
    }
}

struct ResolvedDialog<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    base: Element<'a, Message, Renderer>,
    backdrop_target: Element<'a, Message, Renderer>,
    panel: Element<'a, Message, Renderer>,
    open: bool,
    on_opened: Option<Message>,
    on_close_intent: Option<Message>,
    on_closed: Option<Message>,
    esc_close: bool,
    animate: bool,
    class: <Theme as Catalog>::Class<'a>,
}

struct WidgetState {
    visibility: Animation<bool>,
    now: Instant,
    was_open: bool,
    opened_emitted: bool,
    closed_emitted: bool,
}

impl WidgetState {
    fn new() -> Self {
        Self {
            visibility: Animation::new(false)
                .duration(Duration::from_millis(500))
                .easing(animation::Easing::EaseInOutBack),
            now: Instant::now(),
            was_open: false,
            opened_emitted: false,
            closed_emitted: false,
        }
    }
}

impl<'a, Message, Renderer> ResolvedDialog<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    fn progress(&self, state: &WidgetState) -> f32 {
        if self.animate {
            state.visibility.interpolate(0.0, 1.0, state.now)
        } else if self.open {
            1.0
        } else {
            0.0
        }
    }

    fn is_showing(&self, state: &WidgetState) -> bool {
        let progress = self.progress(state);
        progress > 0.0 || self.open || (self.animate && state.visibility.is_animating(state.now))
    }

    fn is_transitioning(&self, state: &WidgetState) -> bool {
        self.animate && state.visibility.is_animating(state.now)
    }

    fn did_close(&self, state: &WidgetState) -> bool {
        self.animate
            && !self.open
            && state.was_open
            && !state.visibility.is_animating(state.now)
            && self.progress(state) <= f32::EPSILON
    }

    fn did_open(&self, state: &WidgetState) -> bool {
        self.open
            && (!self.animate
                || (!state.visibility.is_animating(state.now)
                    && self.progress(state) >= 1.0 - f32::EPSILON))
    }
}

impl<Message, Renderer> iced_core::Widget<Message, Theme, Renderer>
    for ResolvedDialog<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<WidgetState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(WidgetState::new())
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(&mut [&mut self.base, &mut self.backdrop_target, &mut self.panel]);
    }

    fn size(&self) -> Size<Length> {
        self.base.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) {
        self.base
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let size = tree.children[0].size;
        let overlay_limits = Limits::new(Size::ZERO, size);
        self.backdrop_target.as_widget_mut().layout(
            &mut tree.children[1],
            renderer,
            &overlay_limits.width(Length::Fill).height(Length::Fill),
        );
        self.panel
            .as_widget_mut()
            .layout(&mut tree.children[2], renderer, &overlay_limits);

        let panel = tree.children[2].size;
        tree.size = size;
        tree.children[0].translation = Vector::ZERO;
        tree.children[1].translation = Vector::ZERO;
        tree.children[2].translation = Vector::new(
            (size.width - panel.width) / 2.0,
            (size.height - panel.height) / 2.0,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<WidgetState>();

        if self.open {
            state.was_open = true;
            state.closed_emitted = false;
        } else {
            state.opened_emitted = false;
        }

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = *now;
        }

        let was_animating = state.visibility.is_animating(state.now);
        if self.animate {
            state.visibility.go_mut(self.open, state.now);
        }

        if self.animate {
            let is_animating = state.visibility.is_animating(state.now);

            if matches!(event, Event::Window(window::Event::RedrawRequested(_))) {
                if is_animating {
                    shell.request_redraw();
                }
            } else if !was_animating && is_animating {
                shell.request_redraw();
            }
        }

        if self.did_open(state) && !state.opened_emitted {
            if let Some(message) = &self.on_opened {
                shell.publish(message.clone());
            }
            state.opened_emitted = true;
        }

        if self.did_close(state) && !state.closed_emitted {
            if let Some(message) = &self.on_closed {
                shell.publish(message.clone());
            }
            state.closed_emitted = true;
            state.was_open = false;
            state.opened_emitted = false;
        }

        let mut children = layout.iter(&tree.children).map(|(layout, _)| layout);
        let Some(base_layout) = children.next() else {
            return;
        };
        let Some(backdrop_layout) = children.next() else {
            return;
        };
        let Some(panel_layout) = children.next() else {
            return;
        };
        drop(children);

        if self.is_showing(state) {
            let is_transitioning = self.is_transitioning(state);
            let panel_press = matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    | Event::Touch(touch::Event::FingerPressed { .. })
            ) && cursor.is_over(panel_layout.bounds());

            if !is_transitioning {
                if !panel_press {
                    self.backdrop_target.as_widget_mut().update(
                        &mut tree.children[1],
                        event,
                        backdrop_layout,
                        cursor,
                        renderer,
                        shell,
                        viewport,
                    );
                }

                self.panel.as_widget_mut().update(
                    &mut tree.children[2],
                    event,
                    panel_layout,
                    cursor,
                    renderer,
                    shell,
                    viewport,
                );

                if self.esc_close
                    && let Some(message) = &self.on_close_intent
                    && let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event
                    && matches!(
                        key.as_ref(),
                        keyboard::Key::Named(keyboard::key::Named::Escape)
                    )
                    && !shell.is_event_captured()
                {
                    shell.publish(message.clone());
                    shell.capture_event();
                }
            }

            let in_bounds = match event {
                Event::Mouse(mouse::Event::ButtonPressed(_)) => {
                    cursor.is_over(backdrop_layout.bounds())
                }
                Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                    backdrop_layout.bounds().contains(*position)
                }
                _ => false,
            };

            if in_bounds && !shell.is_event_captured() {
                shell.capture_event();
            }

            return;
        }

        self.base.as_widget_mut().update(
            &mut tree.children[0],
            event,
            base_layout,
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
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<WidgetState>();
        let mut children = layout.iter(&tree.children).map(|(layout, _)| layout);
        let Some(base_layout) = children.next() else {
            return;
        };

        self.base.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            base_layout,
            cursor,
            viewport,
        );

        let progress = self.progress(state);
        if progress <= 0.0 {
            return;
        }

        let Some(_backdrop_layout) = children.next() else {
            return;
        };
        let Some(panel_layout) = children.next() else {
            return;
        };
        let appearance = <Theme as Catalog>::style(theme, &self.class);

        renderer.start_layer(layout.bounds());
        renderer.fill_quad(
            Quad {
                bounds: layout.bounds(),
                ..Quad::default()
            },
            appearance.scrim.scale_alpha(progress),
        );

        let y_offset = (1.0 - progress) * 18.0;
        renderer.with_translation(Vector::new(0.0, y_offset), |renderer| {
            self.panel.as_widget().draw(
                &tree.children[2],
                renderer,
                theme,
                style,
                panel_layout,
                cursor,
                viewport,
            );
        });
        renderer.end_layer();
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<WidgetState>();
        let mut children = layout.iter(&tree.children).map(|(layout, _)| layout);
        let Some(base_layout) = children.next() else {
            return mouse::Interaction::None;
        };
        let Some(backdrop_layout) = children.next() else {
            return mouse::Interaction::None;
        };
        let Some(panel_layout) = children.next() else {
            return mouse::Interaction::None;
        };

        if self.is_showing(state) {
            if self.is_transitioning(state) {
                return mouse::Interaction::None;
            }

            let panel_interaction = self.panel.as_widget().mouse_interaction(
                &tree.children[2],
                panel_layout,
                cursor,
                viewport,
                renderer,
            );

            if panel_interaction != mouse::Interaction::None {
                panel_interaction
            } else {
                self.backdrop_target.as_widget().mouse_interaction(
                    &tree.children[1],
                    backdrop_layout,
                    cursor,
                    viewport,
                    renderer,
                )
            }
        } else {
            self.base.as_widget().mouse_interaction(
                &tree.children[0],
                base_layout,
                cursor,
                viewport,
                renderer,
            )
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
        let mut children = layout.iter_mut(&mut tree.children);

        if let Some((base_layout, base_tree)) = children.next() {
            self.base.as_widget_mut().operate(
                base_tree,
                base_layout,
                viewport,
                renderer,
                operation,
            );
        }

        if let Some((backdrop_layout, backdrop_tree)) = children.next() {
            self.backdrop_target.as_widget_mut().operate(
                backdrop_tree,
                backdrop_layout,
                viewport,
                renderer,
                operation,
            );
        }

        if let Some((panel_layout, panel_tree)) = children.next() {
            self.panel.as_widget_mut().operate(
                panel_tree,
                panel_layout,
                viewport,
                renderer,
                operation,
            );
        }
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
        let show_panel_overlay = {
            let state = tree.state.downcast_ref::<WidgetState>();
            self.is_showing(state) && !self.is_transitioning(state)
        };

        let mut children = layout.iter_mut(&mut tree.children);
        let (Some((base_layout, base_tree)), Some(_), Some((panel_layout, panel_tree))) =
            (children.next(), children.next(), children.next())
        else {
            return Vec::new();
        };

        let mut overlays = self.base.as_widget_mut().overlay(
            base_tree,
            base_layout,
            renderer,
            viewport,
            translation,
            window,
        );

        if show_panel_overlay {
            overlays.extend(self.panel.as_widget_mut().overlay(
                panel_tree,
                panel_layout,
                renderer,
                viewport,
                translation,
                window,
            ));
        }

        overlays
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_no_animation() {
        let mut state = State::default().animated(false);
        state.open();
        assert_eq!(state.status, Status::Open);
    }

    #[test]
    fn open_animated() {
        let mut state = State::default().animated(true);
        state.open();
        assert_eq!(state.status, Status::Opening);
    }

    #[test]
    fn update_close_no_animation() {
        let mut state = State {
            status: Status::Open,
            is_animated: false,
        };
        let action = state.update(Message::Close);
        assert_eq!(state.status, Status::Closed);
        assert_eq!(action, Some(Action::Closed));
    }

    #[test]
    fn update_close_animated() {
        let mut state = State {
            status: Status::Open,
            is_animated: true,
        };
        let action = state.update(Message::Close);
        assert_eq!(state.status, Status::Closing);
        assert_eq!(action, None);

        let action = state.update(Message::Closed);
        assert_eq!(state.status, Status::Closed);
        assert_eq!(action, Some(Action::Closed));
    }
}
