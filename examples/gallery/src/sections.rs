use crate::gallery::Gallery;
use crate::message::Message;
use crate::section::Section;
use moonveil::Element;
use moonveil::Widget;

mod actions;
mod choice;
mod fields;
mod indicators;
pub(super) mod markdown;
mod overlays;
mod overview;
mod selection;
mod surfaces;

pub(super) fn view(gallery: &Gallery) -> Element<'_, Message> {
    match gallery.section {
        Section::Overview => overview::overview(gallery).boxed(),
        Section::Actions => actions::actions().boxed(),
        Section::Indicators => indicators::indicators().boxed(),
        Section::Selection => selection::selection(gallery).boxed(),
        Section::Fields => fields::fields(gallery).boxed(),
        Section::Choice => choice::choice(gallery).boxed(),
        Section::Surfaces => surfaces::surfaces().boxed(),
        Section::Overlays => overlays::overlays(gallery).boxed(),
        Section::Markdown => markdown::markdown_section(gallery).boxed(),
    }
}
