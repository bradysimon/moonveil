//! Tooltips display a hint of information over some element when hovered.
//!
//! By default, the tooltip is displayed immediately, however, this can be adjusted
//! with [`Tooltip::delay`].

use crate::{spacing, widget::container};
use iced_core::{Padding, Pixels, time::Duration};

pub use iced_widget::tooltip::Position;

/// The default gap between a tooltip and its anchor.
pub const DEFAULT_GAP: Pixels = Pixels(spacing::XS);

/// The default padding around tooltip content.
pub const DEFAULT_PADDING: Padding = Padding {
    top: spacing::XS,
    bottom: spacing::XS,
    left: spacing::SM,
    right: spacing::SM,
};

/// The default delay before a tooltip appears.
pub const DEFAULT_DELAY: Duration = Duration::from_millis(400);

pub type Tooltip<W, V> = iced_widget::Tooltip<W, V>;

/// Creates a tooltip using Moonveil's floating surface treatment.
pub fn tooltip<'a, W, V>(
    content: W,
    tooltip: V,
    position: Position,
) -> Tooltip<W, container::Container<'a, V>> {
    Tooltip::new(
        content,
        container::Container::new(tooltip)
            .padding(DEFAULT_PADDING)
            .class(container::Variant::Popover),
    )
    .position(position)
    .gap(DEFAULT_GAP)
    .delay(DEFAULT_DELAY)
}
