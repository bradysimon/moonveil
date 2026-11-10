mod gallery;
mod message;
mod preview_themes;
mod section;
mod sections;
mod shared;
mod ui;

use gallery::Gallery;

fn main() -> iced::Result {
    iced::application(Gallery::new, Gallery::update, Gallery::view)
        .title("Moonveil Gallery")
        .theme(Gallery::theme)
        .run()
}
