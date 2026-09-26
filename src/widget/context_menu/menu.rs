use iced_core::{Point, Rectangle, Size, Vector, mouse};

use super::{Item, State, defaults::*};

/// The inputs used to place every open menu level.
#[derive(Debug, Clone, Copy)]
pub(super) struct Metrics {
    pub width: f32,
    pub max_height: f32,
    /// The overlay bounds that menus must stay within.
    pub viewport: Rectangle,
    /// Converts the widget-local open position into overlay coordinates.
    pub translation: Vector,
}

/// A single placed menu panel: the root menu or an open submenu.
#[derive(Debug, Clone)]
pub(super) struct Level<'a, Message> {
    pub items: &'a [Item<'a, Message>],
    pub path: Vec<usize>,
    pub bounds: Rectangle,
    pub content_height: f32,
    pub scroll_offset: f32,
}

pub(super) fn stack<'a, Message>(
    items: &'a [Item<'a, Message>],
    state: &State,
    metrics: &Metrics,
) -> Vec<Level<'a, Message>> {
    let mut state = state.clone();
    stack_mut(items, &mut state, metrics)
}

/// Places every open menu level and clamps the stored scroll offsets to their content.
pub(super) fn stack_mut<'a, Message>(
    items: &'a [Item<'a, Message>],
    state: &mut State,
    metrics: &Metrics,
) -> Vec<Level<'a, Message>> {
    let Some(open_at) = state.open_at else {
        return Vec::new();
    };

    ensure_scroll_offsets_len(&mut state.scroll_offsets, state.active_path.len() + 1);

    let root = place_root(
        open_at + metrics.translation,
        Size::new(
            metrics.width,
            height(items, metrics.max_height, metrics.viewport.height),
        ),
        metrics.viewport,
    );
    let mut root = Level {
        items,
        path: Vec::new(),
        bounds: root,
        content_height: content_height(items),
        scroll_offset: 0.0,
    };
    clamp_scroll_offset(&mut state.scroll_offsets[0], &root);
    root.scroll_offset = state.scroll_offsets[0];

    let mut levels = vec![root];
    let mut prefer_left = false;

    for index in state.active_path.clone() {
        let Some(parent) = levels.last() else {
            break;
        };
        let Some(children) = parent.items.get(index).and_then(Item::submenu_items) else {
            break;
        };

        let row = item_bounds(parent, index);
        let parent_bounds = parent.bounds;
        let mut path = parent.path.clone();
        path.push(index);

        let submenu_bounds = place_submenu(
            parent_bounds,
            row,
            Size::new(
                metrics.width,
                height(children, metrics.max_height, metrics.viewport.height),
            ),
            metrics.viewport,
            prefer_left,
        );
        prefer_left = submenu_bounds.x < parent_bounds.x;

        let scroll_level = levels.len();
        ensure_scroll_offsets_len(&mut state.scroll_offsets, scroll_level + 1);
        let mut submenu = Level {
            items: children,
            path,
            bounds: submenu_bounds,
            content_height: content_height(children),
            scroll_offset: 0.0,
        };
        clamp_scroll_offset(&mut state.scroll_offsets[scroll_level], &submenu);
        submenu.scroll_offset = state.scroll_offsets[scroll_level];

        levels.push(submenu);
    }

    levels
}

pub(super) fn ensure_scroll_offsets_len(scroll_offsets: &mut Vec<f32>, len: usize) {
    if scroll_offsets.len() < len {
        scroll_offsets.resize(len, 0.0);
    }
}

pub(super) fn height<Message>(
    items: &[Item<'_, Message>],
    max_height: f32,
    viewport_height: f32,
) -> f32 {
    content_height(items)
        .min(max_height)
        .min((viewport_height - EDGE_PADDING * 2.0).max(ITEM_HEIGHT))
}

pub(super) fn content_height<Message>(items: &[Item<'_, Message>]) -> f32 {
    MENU_PADDING * 2.0 + items.iter().map(item_height).sum::<f32>()
}

pub(super) fn item_height<Message>(item: &Item<'_, Message>) -> f32 {
    match item {
        Item::Separator => SEPARATOR_HEIGHT,
        Item::Action { .. } | Item::Submenu { .. } => ITEM_HEIGHT,
    }
}

pub(super) fn item_bounds<Message>(menu: &Level<'_, Message>, index: usize) -> Rectangle {
    let y = menu.bounds.y + MENU_PADDING - menu.scroll_offset
        + menu.items.iter().take(index).map(item_height).sum::<f32>();

    row_bounds(menu, y, item_height(&menu.items[index]))
}

/// Places a row horizontally, keeping it clear of the scrollbar when the menu scrolls.
fn row_bounds<Message>(menu: &Level<'_, Message>, y: f32, height: f32) -> Rectangle {
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

fn is_scrollable<Message>(menu: &Level<'_, Message>) -> bool {
    menu.content_height > menu.bounds.height
}

pub(super) fn content_bounds(menu_bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: menu_bounds.x,
        y: menu_bounds.y + MENU_PADDING,
        width: menu_bounds.width,
        height: (menu_bounds.height - MENU_PADDING * 2.0).max(0.0),
    }
}

pub(super) fn place_root(anchor: Point, size: Size, viewport: Rectangle) -> Rectangle {
    let right = viewport.x + viewport.width - EDGE_PADDING;
    let bottom = viewport.y + viewport.height - EDGE_PADDING;

    Rectangle {
        x: anchor
            .x
            .min(right - size.width)
            .max(viewport.x + EDGE_PADDING),
        y: anchor
            .y
            .min(bottom - size.height)
            .max(viewport.y + EDGE_PADDING),
        width: size.width,
        height: size.height,
    }
}

/// Places a submenu beside its parent, keeping the side chosen by earlier levels when it fits.
pub(super) fn place_submenu(
    parent: Rectangle,
    row: Rectangle,
    size: Size,
    viewport: Rectangle,
    prefer_left: bool,
) -> Rectangle {
    let left_edge = viewport.x + EDGE_PADDING;
    let right = viewport.x + viewport.width - EDGE_PADDING;
    let bottom = viewport.y + viewport.height - EDGE_PADDING;

    let right_x = parent.x + parent.width - SUBMENU_OVERLAP;
    let left_x = parent.x - size.width + SUBMENU_OVERLAP;
    let x = if prefer_left && left_x >= left_edge {
        left_x
    } else if !prefer_left && right_x + size.width <= right {
        right_x
    } else if left_x >= left_edge {
        left_x
    } else {
        right_x
    }
    .min(right - size.width)
    .max(left_edge);

    Rectangle {
        x,
        y: (row.y - MENU_PADDING)
            .min(bottom - size.height)
            .max(viewport.y + EDGE_PADDING),
        width: size.width,
        height: size.height,
    }
}

pub(super) fn max_scroll_offset<Message>(menu: &Level<'_, Message>) -> f32 {
    (menu.content_height - menu.bounds.height).max(0.0)
}

pub(super) fn clamp_scroll_offset<Message>(offset: &mut f32, menu: &Level<'_, Message>) {
    *offset = offset.clamp(0.0, max_scroll_offset(menu));
}

pub(super) fn scroll_delta_y(delta: &mouse::ScrollDelta) -> f32 {
    match delta {
        mouse::ScrollDelta::Lines { y, .. } => -*y * SCROLL_LINE_HEIGHT,
        mouse::ScrollDelta::Pixels { y, .. } => -*y,
    }
}

/// Scrolls one menu level and returns whether its offset changed.
pub(super) fn scroll_by<Message>(
    state: &mut State,
    menus: &[Level<'_, Message>],
    level: usize,
    delta_y: f32,
) -> bool {
    let Some(menu) = menus.get(level) else {
        return false;
    };

    ensure_scroll_offsets_len(&mut state.scroll_offsets, level + 1);

    let previous = state.scroll_offsets[level];
    state.scroll_offsets[level] += delta_y;
    clamp_scroll_offset(&mut state.scroll_offsets[level], menu);

    (state.scroll_offsets[level] - previous).abs() > f32::EPSILON
}

/// Returns the path hovered at `position`; hovering chrome or a disabled row yields its menu path.
pub(super) fn path_at_position<Message>(
    menus: &[Level<'_, Message>],
    position: Point,
) -> Option<Vec<usize>> {
    let menu = menus
        .iter()
        .rev()
        .find(|menu| menu.bounds.contains(position))?;

    let content = content_bounds(menu.bounds);
    let mut path = menu.path.clone();
    if !content.contains(position) {
        return Some(path);
    }

    let mut y = menu.bounds.y + MENU_PADDING - menu.scroll_offset;
    for (index, item) in menu.items.iter().enumerate() {
        let height = item_height(item);
        let bounds = row_bounds(menu, y, height);

        if bounds
            .intersection(&content)
            .is_some_and(|visible| visible.contains(position))
        {
            if item.is_enabled() {
                path.push(index);
            }
            return Some(path);
        }

        y += height;
    }

    Some(path)
}

pub(super) fn contains<Message>(menus: &[Level<'_, Message>], position: Point) -> bool {
    menus.iter().any(|menu| menu.bounds.contains(position))
}

/// Returns the deepest menu level under `position`.
pub(super) fn scroll_level_at_position<Message>(
    menus: &[Level<'_, Message>],
    position: Point,
) -> Option<usize> {
    menus
        .iter()
        .rposition(|menu| menu.bounds.contains(position))
}

/// Returns whether the cursor, moving from `from` to `position`, is heading into the submenu
/// opened from the menu under it.
pub(super) fn is_aiming_at_submenu<Message>(
    menus: &[Level<'_, Message>],
    from: Point,
    position: Point,
) -> bool {
    let Some(level) = scroll_level_at_position(menus, position) else {
        return false;
    };

    menus
        .get(level + 1)
        .is_some_and(|submenu| safety_triangle_contains(from, submenu.bounds, position))
}

/// Tests the triangle from a recent cursor position toward the submenu's near edge.
pub(super) fn safety_triangle_contains(from: Point, submenu: Rectangle, position: Point) -> bool {
    let (apex, edge_x) = if from.x < submenu.x {
        (
            Point::new(from.x - SAFETY_APEX_TOLERANCE, from.y),
            submenu.x,
        )
    } else if from.x > submenu.x + submenu.width {
        (
            Point::new(from.x + SAFETY_APEX_TOLERANCE, from.y),
            submenu.x + submenu.width,
        )
    } else {
        return false;
    };

    point_in_triangle(
        position,
        apex,
        Point::new(edge_x, submenu.y),
        Point::new(edge_x, submenu.y + submenu.height),
    )
}

fn point_in_triangle(point: Point, a: Point, b: Point, c: Point) -> bool {
    if triangle_sign(a, b, c).abs() <= f32::EPSILON {
        return false;
    }

    let d1 = triangle_sign(point, a, b);
    let d2 = triangle_sign(point, b, c);
    let d3 = triangle_sign(point, c, a);

    let has_negative = d1 < -f32::EPSILON || d2 < -f32::EPSILON || d3 < -f32::EPSILON;
    let has_positive = d1 > f32::EPSILON || d2 > f32::EPSILON || d3 > f32::EPSILON;

    !(has_negative && has_positive)
}

fn triangle_sign(a: Point, b: Point, c: Point) -> f32 {
    (a.x - c.x) * (b.y - c.y) - (b.x - c.x) * (a.y - c.y)
}

/// Scrolls every level so its active row is fully visible.
pub(super) fn ensure_active_visible<Message>(
    items: &[Item<'_, Message>],
    state: &mut State,
    metrics: &Metrics,
) {
    let menus = stack_mut(items, state, metrics);

    for (level, menu) in menus.iter().enumerate() {
        let Some(index) = state.active_path.get(level).copied() else {
            break;
        };

        let row = item_bounds(menu, index);
        let top = menu.bounds.y + MENU_PADDING;
        let bottom = menu.bounds.y + menu.bounds.height - MENU_PADDING;

        if row.y < top {
            state.scroll_offsets[level] -= top - row.y;
        } else if row.y + row.height > bottom {
            state.scroll_offsets[level] += row.y + row.height - bottom;
        }

        clamp_scroll_offset(&mut state.scroll_offsets[level], menu);
    }
}

pub(super) fn scrollbar_track_bounds<Message>(menu: &Level<'_, Message>) -> Option<Rectangle> {
    is_scrollable(menu).then(|| Rectangle {
        x: menu.bounds.x + menu.bounds.width - SCROLLBAR_EDGE_PADDING - SCROLLBAR_WIDTH,
        y: menu.bounds.y + SCROLLBAR_EDGE_PADDING,
        width: SCROLLBAR_WIDTH,
        height: (menu.bounds.height - SCROLLBAR_EDGE_PADDING * 2.0).max(0.0),
    })
}

pub(super) fn scrollbar_thumb_bounds<Message>(menu: &Level<'_, Message>) -> Option<Rectangle> {
    let track = scrollbar_track_bounds(menu)?;
    let thumb_height = (menu.bounds.height / menu.content_height * track.height)
        .max(24.0)
        .min(track.height);
    let travel = (track.height - thumb_height).max(0.0);
    let max_offset = max_scroll_offset(menu);
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

pub(super) fn scrollbar_thumb_at_position<Message>(
    menus: &[Level<'_, Message>],
    position: Point,
) -> Option<(usize, Rectangle)> {
    menus.iter().enumerate().rev().find_map(|(level, menu)| {
        let thumb = scrollbar_thumb_bounds(menu)?;
        scrollbar_thumb_hit_bounds(thumb)
            .contains(position)
            .then_some((level, thumb))
    })
}

fn scrollbar_thumb_hit_bounds(thumb: Rectangle) -> Rectangle {
    Rectangle {
        x: thumb.x - SCROLLBAR_HIT_PADDING,
        y: thumb.y - SCROLLBAR_HIT_PADDING / 2.0,
        width: thumb.width + SCROLLBAR_HIT_PADDING * 2.0,
        height: thumb.height + SCROLLBAR_HIT_PADDING,
    }
}

pub(super) fn scroll_offset_from_thumb_position<Message>(
    menu: &Level<'_, Message>,
    thumb_y: f32,
) -> f32 {
    let (Some(track), Some(thumb)) = (scrollbar_track_bounds(menu), scrollbar_thumb_bounds(menu))
    else {
        return 0.0;
    };

    let travel = track.height - thumb.height;
    if travel <= f32::EPSILON {
        return 0.0;
    }

    ((thumb_y - track.y) / travel).clamp(0.0, 1.0) * max_scroll_offset(menu)
}
