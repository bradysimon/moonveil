use iced_core::time::Duration;

pub(super) const DEFAULT_WIDTH: f32 = 220.0;
pub(super) const MIN_WIDTH: f32 = 140.0;
pub(super) const MAX_HEIGHT: f32 = 340.0;
pub(super) const EDGE_PADDING: f32 = 6.0;
pub(super) const MENU_PADDING: f32 = 4.0;
pub(super) const ITEM_HEIGHT: f32 = 28.0;
pub(super) const SEPARATOR_HEIGHT: f32 = 9.0;
pub(super) const ITEM_HORIZONTAL_PADDING: f32 = 10.0;
pub(super) const TRAILING_CONTENT_WIDTH: f32 = 72.0;
pub(super) const CHEVRON_SIZE: f32 = 14.0;
pub(super) const SUBMENU_OVERLAP: f32 = 2.0;
pub(super) const SCROLL_LINE_HEIGHT: f32 = 60.0;
pub(super) const SCROLLBAR_WIDTH: f32 = 4.0;
pub(super) const SCROLLBAR_EDGE_PADDING: f32 = 3.0;
/// The space between a scrollable menu's rows and its scrollbar.
pub(super) const SCROLLBAR_GAP: f32 = 3.0;
pub(super) const SCROLLBAR_HIT_PADDING: f32 = 6.0;
pub(super) const SAFETY_HOVER_DELAY: Duration = Duration::from_millis(250);
/// How many recent cursor positions back the safety triangle apex is taken from.
pub(super) const POINTER_TRAIL_LEN: usize = 3;
/// Widens the safety triangle so small jitter while aiming at a submenu is forgiven.
pub(super) const SAFETY_APEX_TOLERANCE: f32 = 2.0;
