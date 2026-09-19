//! A lightweight overlay for transient in-app notices.
//!
//! Snackbars are owned by external app state and rendered relative to the
//! wrapped content bounds in one of the four corners.

use std::rc::Rc;

use crate::{
    Element, Theme, icon,
    token::{Intent, Surface},
    widget::{button, column, container, row, scrollable, svg, text},
};
use iced_core::{
    Alignment, Animation, Background, Border, Color, Event,
    Length::{Fill, Fit, Shrink},
    Rectangle, Size, Vector, animation,
    layout::{Limits, Node},
    mouse, overlay,
    renderer::{self, Quad},
    time::{Duration, Instant},
    touch,
    widget::{Operation, Tree, tree},
    window,
};
use iced_widget::{keyed_column, space};

const EDGE_PADDING: f32 = 20.0;
const STACK_SPACING: f32 = 12.0;
const DEFAULT_MAX_WIDTH: f32 = 380.0;
const MIN_WIDTH: f32 = 220.0;
const SLIDE_DISTANCE: f32 = 18.0;
const HEADER_SPACING: f32 = 12.0;
const ICON_SIZE: f32 = 32.0;
const INDICATOR_WIDTH: f32 = 4.0;
const ANIMATION_DURATION: Duration = Duration::from_millis(280);
const MAX_STACK_HEIGHT: f32 = 420.0;

/// A stable identifier for a snackbar entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(u64);

/// Messages emitted by the snackbar widget itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    /// Requests dismissal of the snackbar with the given ID.
    Dismiss(Id),
    /// Requests activation of the snackbar action and dismissal of the snackbar.
    Action(Id),
}

/// The visual variant of a snackbar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Variant {
    /// An accent indicator for prominent application feedback.
    #[default]
    Primary,
    /// A neutral indicator for low-emphasis feedback.
    Secondary,
    /// An indicator carrying explicit semantic intent.
    Semantic(Intent),
}

/// The corner in which snackbars should appear.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Position {
    TopLeft,
    TopRight,
    BottomLeft,
    #[default]
    BottomRight,
}

impl Position {
    pub const ALL: [Position; 4] = [
        Position::TopLeft,
        Position::TopRight,
        Position::BottomLeft,
        Position::BottomRight,
    ];

    fn is_top(self) -> bool {
        matches!(self, Position::TopLeft | Position::TopRight)
    }

    fn is_left(self) -> bool {
        matches!(self, Position::TopLeft | Position::BottomLeft)
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Position::TopLeft => "Top Left",
            Position::TopRight => "Top Right",
            Position::BottomLeft => "Bottom Left",
            Position::BottomRight => "Bottom Right",
        })
    }
}

/// An optional action shown in a snackbar.
#[derive(Debug, Clone)]
pub struct Action<Message> {
    label: String,
    message: Message,
}

impl<Message> Action<Message> {
    pub fn new(label: impl Into<String>, message: Message) -> Self {
        Self {
            label: label.into(),
            message,
        }
    }
}

/// A single snackbar entry stored in [`State`].
#[derive(Debug, Clone)]
pub struct Entry<Message> {
    id: Id,
    title: String,
    description: Option<String>,
    variant: Variant,
    action: Option<Action<Message>>,
    created_at: Instant,
}

impl<Message> Entry<Message> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: Id(0),
            title: title.into(),
            description: None,
            variant: Variant::Semantic(Intent::Success),
            action: None,
            created_at: Instant::now(),
        }
    }

    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[must_use]
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    pub fn action(mut self, label: impl Into<String>, message: Message) -> Self {
        self.action = Some(Action::new(label, message));
        self
    }
}

/// External snackbar state owned by the app.
#[derive(Debug, Clone)]
pub struct State<Message> {
    entries: Vec<Entry<Message>>,
    position: Position,
    next_id: u64,
    animating_id: Option<Id>,
}

impl<Message> Default for State<Message> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            position: Position::default(),
            next_id: 1,
            animating_id: None,
        }
    }
}

impl<AppMessage> State<AppMessage> {
    #[must_use]
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn update(&mut self, message: Message) -> Option<AppMessage> {
        match message {
            Message::Dismiss(id) => {
                _ = self.remove(id);
                None
            }
            Message::Action(id) => {
                let index = self.entries.iter().position(|entry| entry.id == id)?;
                let entry = self.entries.remove(index);
                entry.action.map(|action| action.message)
            }
        }
    }

    pub fn push(&mut self, mut entry: Entry<AppMessage>) -> Id {
        let id = Id(self.next_id);
        self.next_id += 1;
        entry.id = id;
        entry.created_at = Instant::now();
        self.entries.push(entry);
        self.animating_id = Some(id);
        id
    }

    pub fn remove(&mut self, id: Id) -> Option<Entry<AppMessage>> {
        let index = self.entries.iter().position(|entry| entry.id == id)?;
        let removed = self.entries.remove(index);

        if self.animating_id == Some(id) {
            self.animating_id = None;
        }

        Some(removed)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.animating_id = None;
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[Entry<AppMessage>] {
        &self.entries
    }
}

/// The resolved visual style of a snackbar card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The resolved card container style.
    pub card: iced_widget::container::Style,
    /// The snackbar title color.
    pub title: Color,
    /// The snackbar description color.
    pub description: Color,
    /// The leading indicator color.
    pub indicator: Color,
}

/// A custom snackbar style function using Moonveil's concrete theme.
pub type StyleFn<'a> = Box<dyn Fn(&Theme) -> Style + 'a>;

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

/// The theme catalog used to style snackbar cards.
pub trait Catalog {
    /// The snackbar style class type.
    type Class<'a>;

    /// Returns the default snackbar class.
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a snackbar class into a style.
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

#[derive(Clone)]
enum SharedClass<'a> {
    Variant(Variant),
    Custom(Rc<dyn Fn(&Theme) -> Style + 'a>),
}

impl<'a> From<Class<'a>> for SharedClass<'a> {
    fn from(class: Class<'a>) -> Self {
        match class {
            Class::Variant(variant) => Self::Variant(variant),
            Class::Custom(style) => Self::Custom(style.into()),
        }
    }
}

impl SharedClass<'_> {
    fn style(&self, theme: &Theme) -> Style {
        match self {
            Self::Variant(variant) => appearance(theme, *variant),
            Self::Custom(style) => style(theme),
        }
    }
}

/// Returns the resolved style for a built-in snackbar variant.
pub fn appearance(theme: &Theme, variant: Variant) -> Style {
    let indicator = match variant {
        Variant::Primary => theme.colors().accent.indicator,
        Variant::Secondary => theme.colors().content.secondary,
        Variant::Semantic(intent) => match intent {
            Intent::Accent => theme.colors().accent.indicator,
            Intent::Success => theme.colors().success.indicator,
            Intent::Warning => theme.colors().warning.indicator,
            Intent::Danger => theme.colors().danger.indicator,
            Intent::Info => theme.colors().info.indicator,
        },
    };

    Style {
        card: iced_widget::container::Style {
            text_color: Some(theme.colors().content.primary.into()),
            background: Some(Background::Color(theme.surface(Surface::Overlay).into())),
            border: Border {
                color: theme.colors().borders.standard.into(),
                width: theme.appearance().border.hairline,
                radius: theme.appearance().radius.md.into(),
            },
            shadow: theme.appearance().shadow.popover.into(),
            ..iced_widget::container::Style::default()
        },
        title: theme.colors().content.primary.into(),
        description: theme.colors().content.muted.into(),
        indicator: indicator.into(),
    }
}

/// Creates a new [`Snackbar`] overlay wrapping the given content.
pub fn snackbar<'a, Message, Renderer>(
    state: &'a State<Message>,
    base: impl Into<Element<'a, Message, Renderer>>,
) -> Snackbar<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    Snackbar::new(state, base)
}

/// A composable snackbar overlay that renders notices over a base element.
pub struct Snackbar<'a, Message, Renderer = iced_widget::Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    base: Element<'a, Message, Renderer>,
    state: &'a State<Message>,
    on_update: Option<Box<dyn Fn(self::Message) -> Message + 'a>>,
    max_width: f32,
    class: Option<Class<'a>>,
}

impl<'a, Message, Renderer> Snackbar<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    pub fn new(state: &'a State<Message>, base: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Self {
            base: base.into(),
            state,
            on_update: None,
            max_width: DEFAULT_MAX_WIDTH,
            class: None,
        }
    }

    #[must_use]
    pub fn on_update(mut self, mapper: impl Fn(self::Message) -> Message + 'a) -> Self {
        self.on_update = Some(Box::new(mapper));
        self
    }

    #[must_use]
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = width.max(MIN_WIDTH);
        self
    }

    /// Overrides every entry's built-in visual variant.
    #[must_use]
    pub fn class(mut self, variant: Variant) -> Self {
        self.class = Some(Class::Variant(variant));
        self
    }

    /// Overrides every entry with a custom resolved style.
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self {
        self.class = Some(Class::Custom(Box::new(style)));
        self
    }
}

fn build_stack<'a, Message, Renderer>(
    state: &'a State<Message>,
    on_update: Option<&dyn Fn(self::Message) -> Message>,
    max_width: f32,
    class: Option<&SharedClass<'a>>,
) -> Element<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + iced_core::svg::Renderer + iced_core::text::Renderer + 'a,
{
    if state.is_empty() {
        return space().into();
    }

    let entries = if state.position().is_top() {
        state.entries().iter().collect::<Vec<_>>()
    } else {
        state.entries().iter().rev().collect::<Vec<_>>()
    };

    let cards = entries.into_iter().map(|entry| {
        let dismiss_message = on_update.map(|map| map(self::Message::Dismiss(entry.id)));
        let action_message = entry
            .action
            .as_ref()
            .and_then(|_| on_update.map(|map| map(self::Message::Action(entry.id))));
        let class = class
            .cloned()
            .unwrap_or(SharedClass::Variant(entry.variant));

        (
            entry.id,
            snackbar_card(
                entry,
                dismiss_message,
                action_message,
                state.animating_id,
                state.position(),
                max_width,
                class,
            ),
        )
    });

    container(scrollable(keyed_column(cards).spacing(STACK_SPACING)).width(max_width))
        .height(Fit.max(MAX_STACK_HEIGHT))
        .into()
}

fn snackbar_card<'a, Message, Renderer>(
    entry: &'a Entry<Message>,
    dismiss_message: Option<Message>,
    action_message: Option<Message>,
    animating_id: Option<Id>,
    position: Position,
    max_width: f32,
    class: SharedClass<'a>,
) -> Element<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + iced_core::svg::Renderer + iced_core::text::Renderer + 'a,
{
    let action_button: Option<Element<'a, Message, Renderer>> =
        entry.action.as_ref().map(|action| {
            button(text(&action.label).size(13))
                .padding([6, 10])
                .class(button::Variant::Secondary)
                .on_press_maybe(action_message.clone())
                .into()
        });

    let dismiss_button: Option<Element<'a, Message, Renderer>> = dismiss_message.map(|message| {
        button(
            svg(icon::close())
                .width(14)
                .height(14)
                .class(svg::Variant::Primary),
        )
        .padding(6)
        .class(button::Variant::Ghost)
        .on_press(message)
        .into()
    });

    let title_class = class.clone();
    let title: Element<'a, Message, Renderer> = text(&entry.title)
        .size(16)
        .style(move |theme| iced_widget::text::Style {
            color: Some(title_class.style(theme).title),
        })
        .into();

    let description_class = class.clone();
    let description: Option<Element<'a, Message, Renderer>> =
        entry.description.as_ref().map(|description| {
            text(description)
                .size(14)
                .style(move |theme| iced_widget::text::Style {
                    color: Some(description_class.style(theme).description),
                })
                .into()
        });

    let header = row([
        icon_badge(entry.variant, class.clone()),
        title,
        space::horizontal().width(Fill).into(),
        dismiss_button.unwrap_or_else(|| space().into()),
    ])
    .spacing(HEADER_SPACING)
    .align_y(Alignment::Center);

    let description: Option<Element<'a, Message, Renderer>> = description.map(|description| {
        container(description)
            .padding(iced_core::padding::left(ICON_SIZE - INDICATOR_WIDTH))
            .into()
    });

    let footer: Option<Element<'a, Message, Renderer>> = action_button.map(|action_button| {
        container(action_button)
            .width(Fill)
            .align_x(iced_core::alignment::Horizontal::Right)
            .padding(iced_core::padding::left(ICON_SIZE + HEADER_SPACING))
            .into()
    });

    let mut content = column([header.into()]).spacing(4);
    if let Some(description) = description {
        content = content.push(description);
    }
    if let Some(footer) = footer {
        content = content.push(footer);
    }

    let card_class = class.clone();
    let card = container(content)
        .padding([12, 14])
        .width(max_width)
        .style(move |theme| {
            let mut style = card_class.style(theme).card;
            style.background = None;
            style.border = Border::default();
            style.shadow = Default::default();
            style
        });

    animated_card(
        accented_card(card, class),
        animating_id == Some(entry.id),
        position,
        entry.created_at,
    )
    .into()
}

fn icon_badge<'a, Message, Renderer>(
    variant: Variant,
    class: SharedClass<'a>,
) -> Element<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + iced_core::svg::Renderer + iced_core::text::Renderer + 'a,
{
    let handle = match variant {
        Variant::Primary | Variant::Secondary => icon::info(),
        Variant::Semantic(Intent::Accent | Intent::Info) => icon::info(),
        Variant::Semantic(Intent::Success) => icon::check(),
        Variant::Semantic(Intent::Warning) => icon::triangle_alert(),
        Variant::Semantic(Intent::Danger) => icon::circle_alert(),
    };

    container(
        svg(handle)
            .width(16)
            .height(16)
            .style(move |theme, _status| iced_widget::svg::Style {
                color: Some(class.style(theme).indicator),
            }),
    )
    .width(ICON_SIZE)
    .height(ICON_SIZE)
    .center_x(Shrink)
    .center_y(Shrink)
    .into()
}

fn animated_card<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
    animate: bool,
    position: Position,
    created_at: Instant,
) -> AnimatedCard<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    AnimatedCard {
        content: content.into(),
        animate,
        position,
        created_at,
    }
}

struct AnimatedCard<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    content: Element<'a, Message, Renderer>,
    animate: bool,
    position: Position,
    created_at: Instant,
}

struct AnimatedCardState {
    visibility: Animation<bool>,
    created_at: Option<Instant>,
    now: Instant,
}

impl AnimatedCardState {
    fn new() -> Self {
        Self {
            visibility: Animation::new(false)
                .duration(ANIMATION_DURATION)
                .easing(animation::Easing::EaseOut),
            created_at: None,
            now: Instant::now(),
        }
    }
}

impl<Message, Renderer> iced_core::Widget<Message, Theme, Renderer>
    for AnimatedCard<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<AnimatedCardState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(AnimatedCardState::new())
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
    }

    fn size(&self) -> Size<iced_core::Length> {
        self.content.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: iced_core::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<AnimatedCardState>();
        let was_animating = state.visibility.is_animating(state.now);

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = *now;
        }

        if self.animate && state.created_at != Some(self.created_at) {
            state.created_at = Some(self.created_at);
            state.visibility = Animation::new(false)
                .duration(ANIMATION_DURATION)
                .easing(animation::Easing::EaseOut);
            state.visibility.go_mut(true, state.now);
        }

        let is_animating = state.visibility.is_animating(state.now);

        if matches!(event, Event::Window(window::Event::RedrawRequested(_))) {
            if is_animating {
                shell.request_redraw();
            }
        } else if !was_animating && is_animating {
            shell.request_redraw();
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
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
        layout: iced_core::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<AnimatedCardState>();
        let progress = if self.animate {
            state.visibility.interpolate(0.0, 1.0, state.now)
        } else {
            1.0
        };

        renderer.with_translation(
            Vector::new(0.0, slide_offset(self.position, progress)),
            |renderer| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor,
                    viewport,
                );
            },
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: iced_core::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
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
        layout: iced_core::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: iced_core::Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Renderer> From<AnimatedCard<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    fn from(card: AnimatedCard<'a, Message, Renderer>) -> Self {
        Element::new(card)
    }
}

fn accented_card<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
    class: SharedClass<'a>,
) -> AccentedCard<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    AccentedCard {
        content: content.into(),
        class,
    }
}

struct AccentedCard<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    content: Element<'a, Message, Renderer>,
    class: SharedClass<'a>,
}

impl<Message, Renderer> iced_core::Widget<Message, Theme, Renderer>
    for AccentedCard<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
    }

    fn size(&self) -> Size<iced_core::Length> {
        self.content.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: iced_core::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
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
        layout: iced_core::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let snackbar_style = self.class.style(theme);
        let bounds = layout.bounds();

        let background = match snackbar_style.card.background {
            Some(Background::Color(card_color)) => {
                let indicator_end = (INDICATOR_WIDTH / bounds.width.max(INDICATOR_WIDTH)).min(1.0);
                let transition_end = (indicator_end + 1.0 / bounds.width.max(1.0)).min(1.0);
                let gradient = iced_core::gradient::Linear::new(std::f32::consts::FRAC_PI_2)
                    .add_stop(0.0, snackbar_style.indicator)
                    .add_stop(indicator_end, snackbar_style.indicator)
                    .add_stop(transition_end, card_color)
                    .add_stop(1.0, card_color);

                Background::Gradient(gradient.into())
            }
            Some(background) => background,
            None => Background::Color(Color::TRANSPARENT),
        };

        renderer.fill_quad(
            Quad {
                bounds,
                border: snackbar_style.card.border,
                shadow: snackbar_style.card.shadow,
                snap: snackbar_style.card.snap,
            },
            background,
        );

        self.content.as_widget().draw(
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
        layout: iced_core::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
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
        layout: iced_core::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: iced_core::Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Renderer> From<AccentedCard<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    fn from(card: AccentedCard<'a, Message, Renderer>) -> Self {
        Element::new(card)
    }
}

fn slide_offset(position: Position, progress: f32) -> f32 {
    let distance = (1.0 - progress.clamp(0.0, 1.0)) * SLIDE_DISTANCE;
    if position.is_top() {
        -distance
    } else {
        distance
    }
}

struct ResolvedSnackbar<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    base: Element<'a, Message, Renderer>,
    stack: Element<'a, Message, Renderer>,
    position: Position,
    has_entries: bool,
}

impl<Message, Renderer> ResolvedSnackbar<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    fn base_cursor(&self, cursor: mouse::Cursor, stack_bounds: Rectangle) -> mouse::Cursor {
        if self.has_entries && cursor.is_over(stack_bounds) {
            cursor.levitate()
        } else {
            cursor
        }
    }

    fn stack_position(&self, content_size: Size, stack_size: Size) -> Vector {
        let x = if self.position.is_left() {
            EDGE_PADDING
        } else {
            (content_size.width - stack_size.width - EDGE_PADDING).max(EDGE_PADDING)
        };
        let y = if self.position.is_top() {
            EDGE_PADDING
        } else {
            (content_size.height - stack_size.height - EDGE_PADDING).max(EDGE_PADDING)
        };

        Vector::new(x, y)
    }
}

impl<'a, Message, Renderer> From<Snackbar<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + iced_core::svg::Renderer + iced_core::text::Renderer + 'a,
{
    fn from(snackbar: Snackbar<'a, Message, Renderer>) -> Self {
        let Snackbar {
            base,
            state,
            on_update,
            max_width,
            class,
        } = snackbar;

        let class = class.map(SharedClass::from);
        let stack = build_stack(state, on_update.as_deref(), max_width, class.as_ref());

        Element::new(ResolvedSnackbar {
            base,
            stack,
            position: state.position(),
            has_entries: !state.is_empty(),
        })
    }
}

impl<Message, Renderer> iced_core::Widget<Message, Theme, Renderer>
    for ResolvedSnackbar<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(&mut [&mut self.base, &mut self.stack]);
    }

    fn size(&self) -> Size<iced_core::Length> {
        self.base.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let base = self
            .base
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let size = base.size();
        let overlay_limits = Limits::new(Size::ZERO, size);
        let mut stack =
            self.stack
                .as_widget_mut()
                .layout(&mut tree.children[1], renderer, &overlay_limits);
        let position = self.stack_position(size, stack.size());
        stack = stack.move_to((position.x, position.y));

        Node::with_children(size, vec![base, stack])
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: iced_core::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let mut children = layout.children();
        let Some(base_layout) = children.next() else {
            return;
        };
        let Some(stack_layout) = children.next() else {
            return;
        };

        if self.has_entries {
            self.stack.as_widget_mut().update(
                &mut tree.children[1],
                event,
                stack_layout,
                cursor,
                renderer,
                shell,
                viewport,
            );

            let in_stack_bounds = match event {
                Event::Mouse(mouse::Event::ButtonPressed(_)) => {
                    cursor.is_over(stack_layout.bounds())
                }
                Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                    stack_layout.bounds().contains(*position)
                }
                _ => false,
            };

            if in_stack_bounds && !shell.is_event_captured() {
                shell.capture_event();
            }
        }

        let base_cursor = self.base_cursor(cursor, stack_layout.bounds());

        if !shell.is_event_captured() {
            self.base.as_widget_mut().update(
                &mut tree.children[0],
                event,
                base_layout,
                base_cursor,
                renderer,
                shell,
                viewport,
            );
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: iced_core::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let mut children = layout.children();
        let Some(base_layout) = children.next() else {
            return;
        };
        let stack_layout = children.next();
        let base_cursor = stack_layout
            .map(|stack_layout| self.base_cursor(cursor, stack_layout.bounds()))
            .unwrap_or(cursor);

        self.base.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            base_layout,
            base_cursor,
            viewport,
        );

        if !self.has_entries {
            return;
        }

        let Some(stack_layout) = stack_layout else {
            return;
        };

        renderer.start_layer(layout.bounds());
        self.stack.as_widget().draw(
            &tree.children[1],
            renderer,
            theme,
            style,
            stack_layout,
            cursor,
            viewport,
        );
        renderer.end_layer();
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: iced_core::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut children = layout.children();
        let Some(base_layout) = children.next() else {
            return mouse::Interaction::None;
        };
        let Some(stack_layout) = children.next() else {
            return mouse::Interaction::None;
        };

        let base_cursor = self.base_cursor(cursor, stack_layout.bounds());

        if self.has_entries {
            let interaction = self.stack.as_widget().mouse_interaction(
                &tree.children[1],
                stack_layout,
                cursor,
                viewport,
                renderer,
            );
            if interaction != mouse::Interaction::None {
                return interaction;
            }

            if cursor.is_over(stack_layout.bounds()) {
                return mouse::Interaction::Idle;
            }
        }

        self.base.as_widget().mouse_interaction(
            &tree.children[0],
            base_layout,
            base_cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: iced_core::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let mut children = layout.children();

        if let Some(base_layout) = children.next() {
            self.base.as_widget_mut().operate(
                &mut tree.children[0],
                base_layout,
                renderer,
                operation,
            );
        }

        if let Some(stack_layout) = children.next() {
            self.stack.as_widget_mut().operate(
                &mut tree.children[1],
                stack_layout,
                renderer,
                operation,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: iced_core::Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let mut children = layout.children();
        let base_layout = children.next()?;
        let stack_layout = children.next()?;

        let (base_tree, stack_tree) = tree.children.split_at_mut(1);
        let base_tree = &mut base_tree[0];
        let stack_tree = &mut stack_tree[0];
        let mut overlays = Vec::new();

        if let Some(base_overlay) = self.base.as_widget_mut().overlay(
            base_tree,
            base_layout,
            renderer,
            viewport,
            translation,
        ) {
            overlays.push(base_overlay);
        }

        if self.has_entries
            && let Some(stack_overlay) = self.stack.as_widget_mut().overlay(
                stack_tree,
                stack_layout,
                renderer,
                viewport,
                translation,
            )
        {
            overlays.push(stack_overlay);
        }

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestMessage {
        Retry,
    }

    #[test]
    fn push_assigns_unique_ids() {
        let mut state = State::<()>::default();

        let first = state.push(Entry::new("First"));
        let second = state.push(Entry::new("Second"));

        assert_ne!(first, second);
        assert_eq!(state.entries.len(), 2);
        assert_eq!(state.entries[0].id, first);
        assert_eq!(state.entries[1].id, second);
    }

    #[test]
    fn remove_deletes_only_matching_entry() {
        let mut state = State::<()>::default();

        let first = state.push(Entry::new("First"));
        let second = state.push(Entry::new("Second"));
        let removed = state.remove(first);

        assert!(removed.is_some());
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.entries[0].id, second);
    }

    #[test]
    fn slide_offset_comes_from_edge() {
        assert!(slide_offset(Position::TopRight, 0.5) < 0.0);
        assert!(slide_offset(Position::BottomLeft, 0.5) > 0.0);
        assert_eq!(slide_offset(Position::BottomRight, 1.0), 0.0);
    }

    #[test]
    fn update_action_dismisses_and_returns_message() {
        let mut state = State::<TestMessage>::default();
        let id = state.push(Entry::new("Retry").action("Retry", TestMessage::Retry));

        let message = state.update(Message::Action(id));

        assert_eq!(message, Some(TestMessage::Retry));
        assert!(state.is_empty());
    }

    #[test]
    fn removing_newest_does_not_retarget_animation_to_older_entry() {
        let mut state = State::<()>::default().with_position(Position::TopRight);

        let first = state.push(Entry::new("First"));
        let second = state.push(Entry::new("Second"));

        assert_eq!(state.animating_id, Some(second));

        state.remove(second);

        assert_eq!(state.animating_id, None);
        assert_eq!(state.entries[0].id, first);
    }
}
