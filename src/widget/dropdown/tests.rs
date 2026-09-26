use super::{defaults::*, menu::*, *};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    One,
    Two,
    Three,
}

fn menu() -> Vec<Item<'static, Message>> {
    vec![
        section("Primary"),
        item("One", Message::One).leading(Leading::text(iced_core::Font::default(), '*')),
        separator(),
        item("Disabled", Message::Two).disabled(true),
        item("Three", Message::Three),
    ]
}

#[test]
fn opening_selects_first_enabled_action() {
    let items = menu();
    let mut state = State::new();
    state.open(&items);
    assert_eq!(state.active_index, Some(1));
}

#[test]
fn keyboard_navigation_skips_headers_separators_and_disabled_items() {
    let items = menu();
    let mut state = State::new();
    state.open(&items);
    state.move_next(&items);
    assert_eq!(state.active_index, Some(4));
    state.move_next(&items);
    assert_eq!(state.active_index, Some(1));
    state.move_previous(&items);
    assert_eq!(state.active_index, Some(4));
}

#[test]
fn disabled_items_have_no_message() {
    let items = menu();
    assert_eq!(items[3].message(), None);
}

#[test]
fn hit_testing_returns_disabled_rows_without_activating_them() {
    let items = menu();
    let trigger = Rectangle::new(Point::new(10.0, 10.0), Size::new(120.0, 30.0));
    let menu = layout(
        &items,
        0.0,
        trigger,
        Some(DEFAULT_WIDTH),
        MAX_HEIGHT,
        4.0,
        Rectangle::with_size(Size::new(800.0, 600.0)),
    );
    assert_eq!(
        hit_test(&items, &menu, item_bounds(&items, 3, &menu).center()),
        Hit::Row(3)
    );
    assert!(!items[3].is_enabled());
}

#[test]
fn hidden_scrolled_rows_do_not_hit_through_padding() {
    let items = (0..8)
        .map(|index| item(format!("Item {index}"), Message::One))
        .collect::<Vec<_>>();
    let trigger = Rectangle::new(Point::new(10.0, 10.0), Size::new(120.0, 30.0));
    let menu = layout(
        &items,
        ITEM_HEIGHT,
        trigger,
        Some(DEFAULT_WIDTH),
        100.0,
        4.0,
        Rectangle::with_size(Size::new(800.0, 600.0)),
    );
    let position = Point::new(
        menu.bounds.x + MENU_PADDING + ITEM_HORIZONTAL_PADDING,
        menu.bounds.y + MENU_PADDING / 2.0,
    );
    assert_eq!(hit_test(&items, &menu, position), Hit::Chrome);
}

#[test]
fn menu_flips_above_trigger_near_viewport_bottom() {
    let items = menu();
    let trigger = Rectangle::new(Point::new(100.0, 560.0), Size::new(120.0, 30.0));
    let menu = layout(
        &items,
        0.0,
        trigger,
        Some(DEFAULT_WIDTH),
        MAX_HEIGHT,
        4.0,
        Rectangle::with_size(Size::new(800.0, 600.0)),
    );
    assert!(menu.bounds.y + menu.bounds.height <= trigger.y);
}

/// Ensures a menu opens below the trigger when possible,
/// even if part of the parent viewport is clipped (e.g. due to scrollable).
#[test]
fn menu_opens_below_when_overlay_has_room_beyond_parent_viewport() {
    let items = menu();
    let trigger = Rectangle::new(Point::new(100.0, 400.0), Size::new(120.0, 30.0));
    let parent_viewport = Rectangle::new(Point::new(0.0, 80.0), Size::new(800.0, 400.0));
    let overlay_viewport = Rectangle::with_size(Size::new(800.0, 600.0));

    let clipped = layout(
        &items,
        0.0,
        trigger,
        Some(DEFAULT_WIDTH),
        MAX_HEIGHT,
        4.0,
        parent_viewport,
    );
    let menu = layout(
        &items,
        0.0,
        trigger,
        Some(DEFAULT_WIDTH),
        MAX_HEIGHT,
        4.0,
        overlay_viewport,
    );

    assert!(clipped.bounds.y + clipped.bounds.height <= trigger.y);
    assert!(menu.bounds.y >= trigger.y + trigger.height);
}

#[test]
fn scroll_offset_is_clamped_to_content() {
    let items = (0..20)
        .map(|index| item(format!("Item {index}"), Message::One))
        .collect::<Vec<_>>();
    let trigger = Rectangle::new(Point::new(10.0, 10.0), Size::new(120.0, 30.0));
    let mut menu = layout(
        &items,
        0.0,
        trigger,
        Some(DEFAULT_WIDTH),
        120.0,
        4.0,
        Rectangle::with_size(Size::new(800.0, 600.0)),
    );
    let mut offset = 10_000.0;
    clamp_scroll_offset(&mut offset, &menu);
    menu.scroll_offset = offset;
    assert_eq!(
        offset,
        max_scroll_offset(menu.content_height, menu.bounds.height)
    );
}

#[test]
fn menu_corners_are_concentric_with_control_corners() {
    let theme = Theme::default_dark();
    let style = menu::appearance(&theme, menu::Variant::Standard);

    assert_eq!(
        style.border.radius.top_left,
        theme.appearance().radius.sm + MENU_PADDING
    );
}
