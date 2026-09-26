use iced_core::{Background, Border, Color, Point, Rectangle, Shadow, Size, mouse};

use super::{Item, defaults::*};
use crate::{
    Radii, Theme,
    token::{Interaction, Surface},
};

/// A styling function for a dropdown menu.
pub type StyleFn<'a> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// The theme catalog of a dropdown menu.
pub trait Catalog {
    type Class<'a>;
    fn default<'a>() -> Self::Class<'a>;
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// The appearance of a dropdown menu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub background: Background,
    pub border: Border,
    pub text_color: Color,
    pub disabled_text_color: Color,
    pub section_text_color: Color,
    pub leading_color: Color,
    pub hover_background: Background,
    pub separator_color: Color,
    pub scrollbar_color: Color,
    pub shadow: Shadow,
}

/// A built-in Moonveil dropdown menu style.
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

/// Returns the resolved style for a built-in menu variant.
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
        section_text_color: theme.colors().content.muted.into(),
        leading_color: theme.colors().content.secondary.into(),
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

#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub bounds: Rectangle,
    pub content_height: f32,
    pub scroll_offset: f32,
    pub has_leading: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    Outside,
    Chrome,
    Row(usize),
}

pub fn layout<Message, Font>(
    items: &[Item<'_, Message, Font>],
    scroll_offset: f32,
    trigger: Rectangle,
    width: Option<f32>,
    max_height: f32,
    offset: f32,
    viewport: Rectangle,
) -> Layout {
    let menu_width = width
        .unwrap_or(trigger.width.max(DEFAULT_WIDTH))
        .max(MIN_WIDTH);
    let content_height = content_height(items);
    let height = content_height
        .min(max_height)
        .min((viewport.height - EDGE_PADDING * 2.0).max(ITEM_HEIGHT));
    let bounds = place_menu(trigger, Size::new(menu_width, height), viewport, offset);
    Layout {
        bounds,
        content_height,
        scroll_offset: scroll_offset.clamp(0.0, max_scroll_offset(content_height, bounds.height)),
        has_leading: items.iter().any(Item::has_leading),
    }
}

pub fn content_bounds(menu: &Layout) -> Rectangle {
    Rectangle {
        x: menu.bounds.x,
        y: menu.bounds.y + MENU_PADDING,
        width: menu.bounds.width,
        height: (menu.bounds.height - MENU_PADDING * 2.0).max(0.0),
    }
}
pub fn content_height<Message, Font>(items: &[Item<'_, Message, Font>]) -> f32 {
    MENU_PADDING * 2.0 + items.iter().map(item_height).sum::<f32>()
}
pub fn item_height<Message, Font>(item: &Item<'_, Message, Font>) -> f32 {
    match item {
        Item::Action { .. } => ITEM_HEIGHT,
        Item::Section { .. } => SECTION_HEIGHT,
        Item::Separator => SEPARATOR_HEIGHT,
    }
}

pub fn place_menu(trigger: Rectangle, size: Size, viewport: Rectangle, offset: f32) -> Rectangle {
    let right = viewport.x + viewport.width - EDGE_PADDING;
    let bottom = viewport.y + viewport.height - EDGE_PADDING;
    let below_y = trigger.y + trigger.height + offset;
    let above_y = trigger.y - size.height - offset;
    let below_space = bottom - below_y;
    let above_space = above_y - (viewport.y + EDGE_PADDING);
    let y = if below_space >= size.height || below_space >= above_space {
        below_y
    } else {
        above_y
    };
    Rectangle {
        x: trigger
            .x
            .min(right - size.width)
            .max(viewport.x + EDGE_PADDING),
        y: y.min(bottom - size.height).max(viewport.y + EDGE_PADDING),
        width: size.width,
        height: size.height,
    }
}

pub fn item_bounds<Message, Font>(
    items: &[Item<'_, Message, Font>],
    index: usize,
    menu: &Layout,
) -> Rectangle {
    let y = menu.bounds.y + MENU_PADDING - menu.scroll_offset
        + items.iter().take(index).map(item_height).sum::<f32>();
    row_bounds(menu, y, item_height(&items[index]))
}

/// Places a row horizontally, keeping it clear of the scrollbar when the menu scrolls.
fn row_bounds(menu: &Layout, y: f32, height: f32) -> Rectangle {
    let right_inset = if is_scrollable(menu) {
        SCROLLBAR_EDGE_PADDING + SCROLLBAR_WIDTH + SCROLLBAR_GAP
    } else {
        MENU_PADDING
    };

    Rectangle {
        x: menu.bounds.x + MENU_PADDING,
        y,
        width: menu.bounds.width - MENU_PADDING - right_inset,
        height,
    }
}

fn is_scrollable(menu: &Layout) -> bool {
    menu.content_height > menu.bounds.height
}

pub fn scrollbar_track_bounds(menu: &Layout) -> Option<Rectangle> {
    is_scrollable(menu).then(|| Rectangle {
        x: menu.bounds.x + menu.bounds.width - SCROLLBAR_EDGE_PADDING - SCROLLBAR_WIDTH,
        y: menu.bounds.y + SCROLLBAR_EDGE_PADDING,
        width: SCROLLBAR_WIDTH,
        height: (menu.bounds.height - SCROLLBAR_EDGE_PADDING * 2.0).max(0.0),
    })
}
pub fn scrollbar_thumb_bounds(menu: &Layout) -> Option<Rectangle> {
    let track = scrollbar_track_bounds(menu)?;
    let thumb_height = (menu.bounds.height / menu.content_height * track.height)
        .max(24.0)
        .min(track.height);
    let travel = (track.height - thumb_height).max(0.0);
    let max_offset = max_scroll_offset(menu.content_height, menu.bounds.height);
    let progress = if max_offset <= f32::EPSILON {
        0.0
    } else {
        menu.scroll_offset / max_offset
    };
    Some(Rectangle {
        x: track.x,
        y: track.y + progress.clamp(0.0, 1.0) * travel,
        width: track.width,
        height: thumb_height,
    })
}
pub fn scrollbar_thumb_at_position(menu: &Layout, position: Point) -> Option<Rectangle> {
    let thumb = scrollbar_thumb_bounds(menu)?;
    scrollbar_thumb_hit_bounds(thumb)
        .contains(position)
        .then_some(thumb)
}
pub fn scrollbar_thumb_hit_bounds(thumb: Rectangle) -> Rectangle {
    Rectangle {
        x: thumb.x - SCROLLBAR_HIT_PADDING,
        y: thumb.y - SCROLLBAR_HIT_PADDING / 2.0,
        width: thumb.width + SCROLLBAR_HIT_PADDING * 2.0,
        height: thumb.height + SCROLLBAR_HIT_PADDING,
    }
}
pub fn scroll_offset_from_thumb_position(menu: &Layout, thumb_y: f32) -> f32 {
    let Some(track) = scrollbar_track_bounds(menu) else {
        return 0.0;
    };
    let Some(thumb) = scrollbar_thumb_bounds(menu) else {
        return 0.0;
    };
    let travel = track.height - thumb.height;
    if travel <= f32::EPSILON {
        return 0.0;
    }
    ((thumb_y - track.y) / travel).clamp(0.0, 1.0)
        * max_scroll_offset(menu.content_height, menu.bounds.height)
}

pub fn hit_test<Message, Font>(
    items: &[Item<'_, Message, Font>],
    menu: &Layout,
    position: Point,
) -> Hit {
    if !menu.bounds.contains(position) {
        return Hit::Outside;
    }
    let content_bounds = content_bounds(menu);
    if !content_bounds.contains(position) {
        return Hit::Chrome;
    }
    let mut y = menu.bounds.y + MENU_PADDING - menu.scroll_offset;
    for (index, item) in items.iter().enumerate() {
        let height = item_height(item);
        let bounds = row_bounds(menu, y, height);
        if bounds
            .intersection(&content_bounds)
            .is_some_and(|visible| visible.contains(position))
        {
            return Hit::Row(index);
        }
        y += height;
    }
    Hit::Chrome
}

pub fn max_scroll_offset(content_height: f32, bounds_height: f32) -> f32 {
    (content_height - bounds_height).max(0.0)
}
pub fn clamp_scroll_offset(offset: &mut f32, menu: &Layout) {
    *offset = offset.clamp(
        0.0,
        max_scroll_offset(menu.content_height, menu.bounds.height),
    );
}
pub fn scroll_delta_y(delta: &mouse::ScrollDelta) -> f32 {
    match delta {
        mouse::ScrollDelta::Lines { y, .. } => -*y * SCROLL_LINE_HEIGHT,
        mouse::ScrollDelta::Pixels { y, .. } => -*y,
    }
}
pub fn scroll_menu_by(offset: &mut f32, menu: &Layout, delta: &mouse::ScrollDelta) -> bool {
    let previous = *offset;
    *offset += scroll_delta_y(delta);
    clamp_scroll_offset(offset, menu);
    (*offset - previous).abs() > f32::EPSILON
}
