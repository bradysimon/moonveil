use super::{defaults::*, menu::*, *};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    One,
    Two,
    Three,
}

fn menu() -> Vec<Item<'static, Message>> {
    vec![
        item("One", Message::One),
        separator(),
        item("Disabled", Message::Two).disabled(true),
        submenu(
            "More",
            [
                item("Two", Message::Two),
                item("Three", Message::Three).shortcut(Shortcut::command(Key::Character('k'))),
            ],
        ),
    ]
}

fn numbered(count: usize) -> Vec<Item<'static, Message>> {
    (0..count)
        .map(|index| item(format!("Item {index}"), Message::One))
        .collect()
}

fn metrics(max_height: f32) -> Metrics {
    Metrics {
        width: DEFAULT_WIDTH,
        max_height,
        viewport: Rectangle::with_size(Size::new(800.0, 600.0)),
        translation: Vector::ZERO,
    }
}

fn opened(items: &[Item<'_, Message>], at: Point) -> State {
    let mut state = State::new();
    state.open(at, items);
    state
}

#[test]
fn keyboard_navigation_skips_separators_and_disabled_items() {
    let items = menu();
    let mut state = opened(&items, Point::ORIGIN);
    assert_eq!(state.active_path, vec![0]);

    state.move_next(&items);
    assert_eq!(state.active_path, vec![3]);

    state.move_previous(&items);
    assert_eq!(state.active_path, vec![0]);
}

#[test]
fn keyboard_navigation_starts_from_ends_when_none_is_active() {
    let items = menu();
    let mut state = opened(&items, Point::ORIGIN);

    state.active_path.clear();
    state.move_next(&items);
    assert_eq!(state.active_path, vec![0]);

    state.active_path.clear();
    state.move_previous(&items);
    assert_eq!(state.active_path, vec![3]);
}

#[test]
fn keyboard_navigation_enters_and_leaves_submenus() {
    let items = menu();
    let mut state = opened(&items, Point::ORIGIN);
    state.move_next(&items);
    state.enter_submenu(&items);
    assert_eq!(state.active_path, vec![3, 0]);

    state.move_next(&items);
    assert_eq!(state.active_path, vec![3, 1]);

    state.leave_submenu();
    assert_eq!(state.active_path, vec![3]);
}

#[test]
fn disabled_submenus_cannot_be_entered() {
    let items = vec![submenu("More", [item("Two", Message::Two)]).disabled(true)];
    let mut state = opened(&items, Point::ORIGIN);
    state.active_path = vec![0];
    state.enter_submenu(&items);

    assert_eq!(state.active_path, vec![0]);
    assert!(!items[0].is_enabled());
}

#[test]
fn shortcuts_find_nested_enabled_actions() {
    let items = menu();
    let key = keyboard::Key::Character("K".into());

    assert_eq!(
        item::shortcut_message(&items, &key, keyboard::Modifiers::COMMAND),
        Some(&Message::Three)
    );
}

#[test]
fn shortcuts_ignore_disabled_actions() {
    let items = vec![
        item("Disabled", Message::One)
            .shortcut(Shortcut::command(Key::Character('d')))
            .disabled(true),
    ];
    let key = keyboard::Key::Character("d".into());

    assert_eq!(
        item::shortcut_message(&items, &key, keyboard::Modifiers::COMMAND),
        None
    );
}

#[test]
fn shortcut_labels_do_not_repeat_the_platform_command_modifier() {
    let label = Shortcut::shift_command(Key::Character('p')).to_string();
    let command = if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    };

    assert_eq!(label.matches(command).count(), 1);
    assert!(label.contains("Shift+"));
    assert!(label.ends_with("+P"));
}

#[test]
fn disabled_items_clear_hover_path_instead_of_becoming_active() {
    let items = menu();
    let state = opened(&items, Point::new(10.0, 10.0));
    let menus = stack(&items, &state, &metrics(MAX_HEIGHT));
    let disabled = item_bounds(&menus[0], 2);

    assert_eq!(path_at_position(&menus, disabled.center()), Some(vec![]));
}

#[test]
fn scrolled_hidden_rows_do_not_hover_through_menu_padding() {
    let items = numbered(8);
    let mut state = opened(&items, Point::new(10.0, 10.0));
    state.scroll_offsets = vec![ITEM_HEIGHT];
    let menus = stack(&items, &state, &metrics(100.0));
    let padding = Point::new(
        menus[0].bounds.x + MENU_PADDING + ITEM_HORIZONTAL_PADDING,
        menus[0].bounds.y + MENU_PADDING / 2.0,
    );

    assert_eq!(path_at_position(&menus, padding), Some(vec![]));
}

#[test]
fn menu_opens_at_translated_position() {
    let items = menu();
    let state = opened(&items, Point::new(20.0, 30.0));
    let metrics = Metrics {
        translation: Vector::new(100.0, 50.0),
        ..metrics(MAX_HEIGHT)
    };
    let menus = stack(&items, &state, &metrics);

    assert_eq!(menus[0].bounds.position(), Point::new(120.0, 80.0));
}

#[test]
fn root_menu_stays_within_viewport() {
    let items = menu();
    let state = opened(&items, Point::new(790.0, 590.0));
    let menus = stack(&items, &state, &metrics(MAX_HEIGHT));
    let bounds = menus[0].bounds;

    assert!(bounds.x + bounds.width <= 800.0 - EDGE_PADDING);
    assert!(bounds.y + bounds.height <= 600.0 - EDGE_PADDING);
}

#[test]
fn submenu_aligns_first_row_with_parent_row() {
    let items = menu();
    let mut state = opened(&items, Point::new(10.0, 10.0));
    state.active_path = vec![3];
    let menus = stack(&items, &state, &metrics(MAX_HEIGHT));
    let parent_row = item_bounds(&menus[0], 3);
    let child_row = item_bounds(&menus[1], 0);

    assert_eq!(parent_row.y, child_row.y);
    assert!(menus[1].bounds.x > menus[0].bounds.x);
}

#[test]
fn submenu_flips_left_near_right_edge() {
    let items = menu();
    let mut state = opened(&items, Point::new(700.0, 10.0));
    state.active_path = vec![3];
    let menus = stack(&items, &state, &metrics(MAX_HEIGHT));

    assert!(menus[1].bounds.x < menus[0].bounds.x);
}

#[test]
fn safety_triangle_holds_motion_toward_right_opening_submenu() {
    let submenu = Rectangle::new(Point::new(198.0, 96.0), Size::new(120.0, 220.0));
    let from = Point::new(150.0, 110.0);

    assert!(safety_triangle_contains(
        from,
        submenu,
        Point::new(160.0, 118.0)
    ));
    assert!(!safety_triangle_contains(
        from,
        submenu,
        Point::new(150.0, 98.0)
    ));
    assert!(!safety_triangle_contains(
        from,
        submenu,
        Point::new(150.0, 130.0)
    ));
    assert!(!safety_triangle_contains(
        from,
        submenu,
        Point::new(140.0, 112.0)
    ));
}

#[test]
fn safety_triangle_holds_motion_toward_left_opening_submenu() {
    let submenu = Rectangle::new(Point::new(102.0, 96.0), Size::new(120.0, 220.0));
    let from = Point::new(270.0, 110.0);

    assert!(safety_triangle_contains(
        from,
        submenu,
        Point::new(260.0, 118.0)
    ));
    assert!(!safety_triangle_contains(
        from,
        submenu,
        Point::new(270.0, 98.0)
    ));
    assert!(!safety_triangle_contains(
        from,
        submenu,
        Point::new(280.0, 112.0)
    ));
}

#[test]
fn moving_vertically_to_a_sibling_is_not_aiming_at_submenu() {
    let items = vec![submenu("More", numbered(5)), item("Other", Message::Two)];
    let mut state = opened(&items, Point::new(100.0, 100.0));
    state.active_path = vec![0];
    let menus = stack(&items, &state, &metrics(MAX_HEIGHT));
    let trigger = item_bounds(&menus[0], 0);
    let sibling = item_bounds(&menus[0], 1);
    let from = trigger.center();

    assert!(!is_aiming_at_submenu(
        &menus,
        from,
        Point::new(from.x, sibling.center_y())
    ));
    assert!(is_aiming_at_submenu(
        &menus,
        from,
        Point::new(from.x + 40.0, sibling.center_y())
    ));
}

#[test]
fn pointer_trail_returns_oldest_recent_position() {
    let mut state = State::new();

    assert_eq!(state.record_pointer(Point::new(0.0, 0.0)), None);
    for x in 1..POINTER_TRAIL_LEN {
        assert_eq!(
            state.record_pointer(Point::new(x as f32, 0.0)),
            Some(Point::ORIGIN)
        );
    }
    assert_eq!(
        state.record_pointer(Point::new(POINTER_TRAIL_LEN as f32, 0.0)),
        Some(Point::ORIGIN)
    );
    assert_eq!(
        state.record_pointer(Point::new(POINTER_TRAIL_LEN as f32 + 1.0, 0.0)),
        Some(Point::new(1.0, 0.0))
    );
}

#[test]
fn safety_hover_delay_keeps_current_path_until_timeout() {
    let mut state = State::new();
    state.active_path = vec![0, 2];
    state.track_pending_safety_hover(vec![1]);
    let start = Instant::now();
    let until = start + SAFETY_HOVER_DELAY;

    assert_eq!(
        state.advance_pending_safety_hover(start),
        SafetyHoverAdvance::Waiting { until }
    );
    assert_eq!(
        state.advance_pending_safety_hover(start + SAFETY_HOVER_DELAY / 2),
        SafetyHoverAdvance::Waiting { until }
    );
    assert_eq!(state.active_path, vec![0, 2]);

    assert_eq!(
        state.advance_pending_safety_hover(until),
        SafetyHoverAdvance::Activated
    );
    assert_eq!(state.active_path, vec![1]);
    assert_eq!(
        state.advance_pending_safety_hover(until),
        SafetyHoverAdvance::Idle
    );
}

#[test]
fn safety_hover_delay_restarts_for_new_candidate() {
    let mut state = State::new();
    state.active_path = vec![0, 2];
    state.track_pending_safety_hover(vec![1]);
    let start = Instant::now();
    let _ = state.advance_pending_safety_hover(start);

    state.track_pending_safety_hover(vec![3]);

    let restarted = start + SAFETY_HOVER_DELAY;
    assert_eq!(
        state.advance_pending_safety_hover(restarted),
        SafetyHoverAdvance::Waiting {
            until: restarted + SAFETY_HOVER_DELAY
        }
    );
    assert_eq!(state.active_path, vec![0, 2]);
    assert_eq!(
        state.advance_pending_safety_hover(restarted + SAFETY_HOVER_DELAY),
        SafetyHoverAdvance::Activated
    );
    assert_eq!(state.active_path, vec![3]);
}

#[test]
fn scroll_target_prefers_deepest_menu_under_cursor() {
    let items = vec![submenu("More", numbered(20))];
    let mut state = opened(&items, Point::new(100.0, 100.0));
    state.active_path = vec![0];
    let menus = stack(&items, &state, &metrics(120.0));

    assert_eq!(
        scroll_level_at_position(&menus, menus[1].bounds.center()),
        Some(1)
    );
}

#[test]
fn menu_background_counts_as_inside_and_scrolls() {
    let items = numbered(20);
    let state = opened(&items, Point::new(10.0, 10.0));
    let menus = stack(&items, &state, &metrics(100.0));
    let background = Point::new(
        menus[0].bounds.x + MENU_PADDING / 2.0,
        menus[0].bounds.center_y(),
    );

    assert!(contains(&menus, background));
    assert_eq!(scroll_level_at_position(&menus, background), Some(0));
}

#[test]
fn keyboard_navigation_scrolls_nested_active_item_into_view() {
    let items = vec![submenu("More", numbered(20))];
    let mut state = opened(&items, Point::new(10.0, 10.0));
    state.active_path = vec![0, 15];
    let metrics = metrics(100.0);

    ensure_active_visible(&items, &mut state, &metrics);
    assert!(state.scroll_offsets[1] > 0.0);

    let menus = stack(&items, &state, &metrics);
    let submenu = &menus[1];
    let row = item_bounds(submenu, 15);

    assert!(row.y >= submenu.bounds.y + MENU_PADDING);
    assert!(row.y + row.height <= submenu.bounds.y + submenu.bounds.height - MENU_PADDING);
}

#[test]
fn scrollbar_drag_position_maps_to_scroll_offset() {
    let items = numbered(20);
    let state = opened(&items, Point::new(10.0, 10.0));
    let menus = stack(&items, &state, &metrics(100.0));
    let menu = &menus[0];
    let track = scrollbar_track_bounds(menu).expect("scrollbar track");
    let thumb = scrollbar_thumb_bounds(menu).expect("scrollbar thumb");

    let offset = scroll_offset_from_thumb_position(menu, track.y + track.height - thumb.height);

    assert!((offset - max_scroll_offset(menu)).abs() < f32::EPSILON);
}

#[test]
fn wheel_scroll_delta_matches_iced_scroll_direction() {
    assert_eq!(
        scroll_delta_y(&mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 }),
        SCROLL_LINE_HEIGHT
    );
    assert_eq!(
        scroll_delta_y(&mouse::ScrollDelta::Pixels { x: 0.0, y: -12.0 }),
        12.0
    );
}

#[test]
fn scroll_by_updates_nested_submenu_offset() {
    let items = vec![submenu("More", numbered(20))];
    let mut state = opened(&items, Point::new(10.0, 10.0));
    state.active_path = vec![0];
    let menus = stack(&items, &state, &metrics(100.0));

    assert!(scroll_by(&mut state, &menus, 1, SCROLL_LINE_HEIGHT));
    assert_eq!(state.scroll_offsets[1], SCROLL_LINE_HEIGHT);
    assert!(scroll_by(&mut state, &menus, 1, -SCROLL_LINE_HEIGHT * 10.0));
    assert_eq!(state.scroll_offsets[1], 0.0);
    assert!(!scroll_by(&mut state, &menus, 1, -SCROLL_LINE_HEIGHT));
}

#[test]
fn stack_preserves_deep_submenu_scroll_offset() {
    let items = vec![submenu("Convert", [submenu("Export As", numbered(20))])];
    let mut state = opened(&items, Point::new(10.0, 10.0));
    state.active_path = vec![0, 0];
    let metrics = metrics(100.0);
    let menus = stack(&items, &state, &metrics);

    assert!(scroll_by(&mut state, &menus, 2, SCROLL_LINE_HEIGHT));
    let menus = stack_mut(&items, &mut state, &metrics);

    assert_eq!(state.scroll_offsets[2], SCROLL_LINE_HEIGHT);
    assert_eq!(menus[2].scroll_offset, SCROLL_LINE_HEIGHT);
}

#[test]
fn scrollable_rows_leave_a_gap_before_the_scrollbar() {
    let items = numbered(20);
    let state = opened(&items, Point::new(10.0, 10.0));
    let menus = stack(&items, &state, &metrics(100.0));
    let track = scrollbar_track_bounds(&menus[0]).expect("scrollbar track");
    let row = item_bounds(&menus[0], 0);

    assert!(row.x + row.width + SCROLLBAR_GAP <= track.x + f32::EPSILON);
}

#[test]
fn non_scrollable_rows_use_symmetric_padding() {
    let items = numbered(3);
    let state = opened(&items, Point::new(10.0, 10.0));
    let menus = stack(&items, &state, &metrics(MAX_HEIGHT));
    let row = item_bounds(&menus[0], 0);

    assert_eq!(row.x - menus[0].bounds.x, MENU_PADDING);
    assert_eq!(
        menus[0].bounds.x + menus[0].bounds.width - (row.x + row.width),
        MENU_PADDING
    );
}

#[test]
fn scrollbar_thumb_hit_test_allows_horizontal_slop() {
    let items = numbered(20);
    let state = opened(&items, Point::new(10.0, 10.0));
    let menus = stack(&items, &state, &metrics(100.0));
    let thumb = scrollbar_thumb_bounds(&menus[0]).expect("scrollbar thumb");
    let position = Point::new(thumb.x - SCROLLBAR_HIT_PADDING + 1.0, thumb.center_y());

    assert_eq!(
        scrollbar_thumb_at_position(&menus, position).map(|(level, _)| level),
        Some(0)
    );
}

#[test]
fn menu_corners_are_concentric_with_control_corners() {
    let theme = Theme::default_dark();
    let style = appearance(&theme, Variant::Standard);

    assert_eq!(
        style.border.radius.top_left,
        theme.appearance().radius.sm + MENU_PADDING
    );
}
