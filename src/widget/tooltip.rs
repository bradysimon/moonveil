//! Tooltips display a hint of information over some element when hovered.
//!
//! By default, the tooltip is displayed immediately, however, this can be adjusted
//! with [`Tooltip::delay`].

use crate::{Element, Theme, spacing, widget::container};
use iced_core::{Pixels, text, time::Duration};

pub use iced_widget::tooltip::Position;

/// The default gap between a tooltip and its anchor.
pub const DEFAULT_GAP: Pixels = Pixels(spacing::XS);

/// The default padding around tooltip content.
pub const DEFAULT_PADDING: Pixels = Pixels(spacing::XS);

/// The default delay before a tooltip appears.
pub const DEFAULT_DELAY: Duration = Duration::from_millis(400);

pub type Tooltip<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::Tooltip<'a, Message, Theme, Renderer>;

/// Creates a tooltip using Moonveil's floating surface treatment.
pub fn tooltip<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
    tooltip: impl Into<Element<'a, Message, Renderer>>,
    position: Position,
) -> Tooltip<'a, Message, Renderer>
where
    Renderer: text::Renderer,
{
    Tooltip::new(content, tooltip, position)
        .class(container::Variant::Popover)
        .gap(DEFAULT_GAP)
        .padding(DEFAULT_PADDING)
        .delay(DEFAULT_DELAY)
}
