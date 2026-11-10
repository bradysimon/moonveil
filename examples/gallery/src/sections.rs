use crate::gallery::Gallery;
use crate::message::Message;
use crate::section::Section;
use moonveil::Element;

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
        Section::Overview => overview::overview(gallery),
        Section::Actions => actions::actions(),
        Section::Indicators => indicators::indicators(),
        Section::Selection => selection::selection(gallery),
        Section::Fields => fields::fields(gallery),
        Section::Choice => choice::choice(gallery),
        Section::Surfaces => surfaces::surfaces(),
        Section::Overlays => overlays::overlays(gallery),
        Section::Markdown => markdown::markdown_section(gallery),
    }
}
