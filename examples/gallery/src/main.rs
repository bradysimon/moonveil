use iced::Length::Fill;
use iced::widget::{column, row};
use iced::{Alignment, Color as IcedColor, Length};
use moonveil::widget::text::TextExt as _;
use moonveil::{
    Color, Element, Radii, Theme, spacing,
    token::Intent,
    widget::{
        Column, Labeled, Space, button, checkbox, container, context_menu, dialog, dropdown,
        labeled, pick_list, progress_bar, radio, rule, scrollable, slider, snackbar, svg, tab_bar,
        text, text_editor, text_input, toggler, tooltip,
    },
};

mod preview_themes;

const COLOR_MODE_ICON: &[u8] = br##"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
    <path d="M15 42 27 17l8 18 5-10 9 17H15Z" fill="#8dd2ff"/>
    <circle cx="45" cy="18" r="7" fill="#ffd27a"/>
    <path d="M18 46h30" stroke="#a6e89a" stroke-width="4" stroke-linecap="round"/>
</svg>
"##;

struct Gallery {
    theme: Theme,
    theme_choice: ThemeChoice,
    rounding: Rounding,
    section: Section,
    include_prereleases: bool,
    accepted_terms: bool,
    environment: Environment,
    preview_tab: PreviewTab,
    auto_retry: bool,
    incident_alerts: bool,
    zoom: f32,
    alert_threshold: f32,
    request_name: String,
    search_query: String,
    endpoint: String,
    code_value: String,
    notes: text_editor::Content,
    snackbars: snackbar::State<Message>,
    dialog: dialog::State,
}

fn main() -> iced::Result {
    iced::application(boot, update, view)
        .title("Moonveil Gallery")
        .theme(|gallery: &Gallery| gallery.theme.clone())
        .run()
}

fn boot() -> Gallery {
    Gallery {
        theme: ThemeChoice::Dark.resolve(Rounding::Standard),
        theme_choice: ThemeChoice::Dark,
        rounding: Rounding::Standard,
        section: Section::Overview,
        include_prereleases: true,
        accepted_terms: false,
        environment: Environment::Staging,
        preview_tab: PreviewTab::Overview,
        auto_retry: true,
        incident_alerts: true,
        zoom: 100.0,
        alert_threshold: 80.0,
        request_name: String::from("Production deploy"),
        search_query: String::new(),
        endpoint: String::from("ftp://invalid.example"),
        code_value: String::from("MOONVEIL_LOG=debug"),
        notes: text_editor::Content::with_text(
            "[deploy]\nregion = \"us-east-1\"\nstrategy = \"rolling\"",
        ),
        snackbars: snackbar::State::default(),
        dialog: dialog::State::default(),
    }
}

#[derive(Debug, Clone)]
enum Message {
    SelectTheme(ThemeChoice),
    SelectRounding(Rounding),
    SelectSection(Section),
    Action,
    ShowSnackbar(snackbar::Variant),
    Snackbar(snackbar::Message),
    OpenDialog,
    Dialog(dialog::Message),
    ConfirmDeployment,
    IncludePrereleases(bool),
    AcceptTerms(bool),
    SelectEnvironment(Environment),
    SelectPreviewTab(PreviewTab),
    AutoRetry(bool),
    IncidentAlerts(bool),
    ZoomChanged(f32),
    AlertThresholdChanged(f32),
    EditField(Field, String),
    EditNotes(text_editor::Action),
}

fn update(gallery: &mut Gallery, message: Message) {
    match message {
        Message::SelectTheme(choice) => {
            gallery.theme = choice.resolve(gallery.rounding);
            gallery.theme_choice = choice;
        }
        Message::SelectRounding(rounding) => {
            gallery.theme = gallery.theme_choice.resolve(rounding);
            gallery.rounding = rounding;
        }
        Message::SelectSection(section) => gallery.section = section,
        Message::Action => {}
        Message::ShowSnackbar(variant) => {
            gallery.snackbars.push(
                snackbar::Entry::new("Deployment update")
                    .description("The preview deployment is ready for review.")
                    .variant(variant)
                    .action("View log", Message::Action),
            );
        }
        Message::Snackbar(message) => {
            _ = gallery.snackbars.update(message);
        }
        Message::OpenDialog => gallery.dialog.open(),
        Message::Dialog(message) => {
            _ = gallery.dialog.update(message);
        }
        Message::ConfirmDeployment => {
            gallery.dialog.close();
        }
        Message::IncludePrereleases(value) => gallery.include_prereleases = value,
        Message::AcceptTerms(value) => gallery.accepted_terms = value,
        Message::SelectEnvironment(environment) => gallery.environment = environment,
        Message::SelectPreviewTab(tab) => gallery.preview_tab = tab,
        Message::AutoRetry(value) => gallery.auto_retry = value,
        Message::IncidentAlerts(value) => gallery.incident_alerts = value,
        Message::ZoomChanged(value) => gallery.zoom = value,
        Message::AlertThresholdChanged(value) => gallery.alert_threshold = value,
        Message::EditField(field, value) => match field {
            Field::RequestName => gallery.request_name = value,
            Field::Search => gallery.search_query = value,
            Field::Endpoint => gallery.endpoint = value,
            Field::CodeValue => gallery.code_value = value,
        },
        Message::EditNotes(action) => gallery.notes.perform(action),
    }
}

#[derive(Debug, Clone, Copy)]
enum Field {
    RequestName,
    Search,
    Endpoint,
    CodeValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    const ALL: [Self; 3] = [Self::Development, Self::Staging, Self::Production];

    fn label(self) -> &'static str {
        match self {
            Self::Development => "Development",
            Self::Staging => "Staging",
            Self::Production => "Production",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreviewTab {
    Overview,
    Activity,
    Settings,
}

impl PreviewTab {
    const ALL: [Self; 3] = [Self::Overview, Self::Activity, Self::Settings];
}

impl std::fmt::Display for PreviewTab {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Overview => "Overview",
            Self::Activity => "Activity",
            Self::Settings => "Settings",
        })
    }
}

fn view(gallery: &Gallery) -> Element<'_, Message> {
    let base = container(row![
        sidebar(gallery),
        rule::vertical(1).class(rule::Variant::Subtle),
        workspace(gallery)
    ])
    .class(container::Variant::Canvas)
    .width(Fill)
    .height(Fill);

    let dialog_config = gallery.dialog.is_visible().then(|| {
        dialog::Config::new(
            text("This will promote the current configuration to production.\n\nThe deployment can be monitored from the activity log.")
                    .class(text::Variant::Secondary)
        )
        .title("Confirm deployment")
        .push_action(
            button(text("Keep editing"))
                .class(button::Variant::Ghost)
                .on_press(Message::Dialog(dialog::Message::Close)),
        )
        .push_action(
            button(text("Deploy now"))
                .class(button::Variant::Primary)
                .on_press(Message::ConfirmDeployment),
        )
    });

    snackbar(
        &gallery.snackbars,
        dialog(base, &gallery.dialog, dialog_config).on_update(Message::Dialog),
    )
    .on_update(Message::Snackbar)
    .into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeChoice {
    Dark,
    Light,
    HighContrast,
    Everforest,
    RosePine,
    CatppuccinLatte,
    Nord,
}

impl ThemeChoice {
    const ALL: [Self; 7] = [
        Self::Dark,
        Self::Light,
        Self::HighContrast,
        Self::Everforest,
        Self::RosePine,
        Self::CatppuccinLatte,
        Self::Nord,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::HighContrast => "High contrast",
            Self::Everforest => "Everforest Light",
            Self::RosePine => "Rosé Pine",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::Nord => "Nord",
        }
    }

    fn resolve(self, rounding: Rounding) -> Theme {
        let theme = match self {
            Self::Dark => Theme::default_dark(),
            Self::Light => Theme::default_light(),
            Self::HighContrast => Theme::default_high_contrast(),
            Self::Everforest => preview_themes::everforest(),
            Self::RosePine => preview_themes::rose_pine(),
            Self::CatppuccinLatte => preview_themes::catppuccin_latte(),
            Self::Nord => preview_themes::nord(),
        };
        let radius = rounding.radii();

        if theme.definition().radius == radius {
            return theme;
        }

        Theme::new(theme.definition().clone().with_radius(radius))
            .expect("changing radii must not affect color resolution")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rounding {
    Sharp,
    Standard,
    Round,
}

impl Rounding {
    const ALL: [Self; 3] = [Self::Sharp, Self::Standard, Self::Round];

    fn label(self) -> &'static str {
        match self {
            Self::Sharp => "Sharp",
            Self::Standard => "Standard",
            Self::Round => "Round",
        }
    }

    fn radii(self) -> Radii {
        match self {
            Self::Sharp => Radii::SHARP,
            Self::Standard => Radii::STANDARD,
            Self::Round => Radii::ROUND,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Overview,
    Actions,
    Indicators,
    Selection,
    Fields,
    Choice,
    Surfaces,
    Overlays,
}

impl Section {
    const ALL: [Self; 8] = [
        Self::Overview,
        Self::Actions,
        Self::Indicators,
        Self::Selection,
        Self::Fields,
        Self::Choice,
        Self::Surfaces,
        Self::Overlays,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Actions => "Actions",
            Self::Indicators => "Rules & progress",
            Self::Selection => "Controls & selection",
            Self::Fields => "Fields & editor",
            Self::Choice => "Choice & scrolling",
            Self::Surfaces => "Surfaces & type",
            Self::Overlays => "Overlays & menus",
        }
    }
}

fn sidebar(gallery: &Gallery) -> Element<'_, Message> {
    let mut navigation = Column::new().spacing(spacing::XS);

    for section in Section::ALL {
        let variant = if section == gallery.section {
            button::Variant::IconSelected
        } else {
            button::Variant::Icon
        };

        navigation = navigation.push(
            button(text(section.label()))
                .class(variant)
                .on_press(Message::SelectSection(section))
                .padding([10.0, spacing::MD])
                .width(Fill),
        );
    }

    container(
        column![
            column![
                text("MOONVEIL").size(13).class(text::Variant::Muted),
                text("Widget gallery")
                    .size(text::size::DISPLAY)
                    .class(text::Variant::Primary)
                    .line_height(1.0),
            ],
            Space::new().height(12),
            text("SECTIONS")
                .size(text::size::CAPTION)
                .class(text::Variant::Muted),
            navigation,
        ]
        .spacing(spacing::MD),
    )
    .class(container::Variant::Canvas)
    .padding(20)
    .width(230)
    .height(Fill)
    .into()
}

fn workspace(gallery: &Gallery) -> Element<'_, Message> {
    container(column![
        toolbar(gallery),
        rule::horizontal(1).class(rule::Variant::Subtle),
        scrollable(section(gallery))
    ])
    .class(container::Variant::Surface)
    .width(Fill)
    .height(Fill)
    .into()
}

fn toolbar(gallery: &Gallery) -> Element<'_, Message> {
    let theme_picker = pick_list(Some(gallery.theme_choice), ThemeChoice::ALL, |choice| {
        choice.label().to_string()
    })
    .class(pick_list::Variant::Standard)
    .on_select(Message::SelectTheme)
    .width(220);
    let rounding_picker = pick_list(Some(gallery.rounding), Rounding::ALL, |rounding| {
        rounding.label().to_string()
    })
    .class(pick_list::Variant::Standard)
    .on_select(Message::SelectRounding)
    .width(140);

    container(
        row![
            column![
                text(gallery.section.label())
                    .size(text::size::HEADING)
                    .class(text::Variant::Primary),
                text(gallery.theme.definition().metadata.name.as_str())
                    .size(text::size::LABEL)
                    .class(text::Variant::Muted),
            ]
            .spacing(3),
            Space::new().width(Fill),
            labeled("Rounding", rounding_picker).reserve_message(false),
            labeled("Theme", theme_picker).reserve_message(false),
        ]
        .spacing(spacing::SM)
        .align_y(Alignment::End),
    )
    .style(toolbar_style)
    .padding([14, 20])
    .width(Fill)
    .into()
}

fn toolbar_style(theme: &Theme) -> container::Style {
    let mut style = container::appearance(theme, container::Variant::Raised);
    style.border.radius = 0.0.into();
    style.border.color = IcedColor::TRANSPARENT;
    style.shadow = iced::Shadow::default();
    style
}

fn section(gallery: &Gallery) -> Element<'_, Message> {
    match gallery.section {
        Section::Overview => overview(gallery),
        Section::Actions => actions(),
        Section::Indicators => indicators(),
        Section::Selection => selection(gallery),
        Section::Fields => fields(gallery),
        Section::Choice => choice(gallery),
        Section::Surfaces => surfaces(),
        Section::Overlays => overlays(gallery),
    }
}

fn overview(gallery: &Gallery) -> Element<'_, Message> {
    let definition = gallery.theme.definition();
    let seed = definition.seed;
    let colors = gallery.theme.colors();
    let description = definition
        .metadata
        .description
        .as_deref()
        .unwrap_or("A contrast-aware palette resolved seed colors.");

    container(
        column![
            container(
                column![
                    text("ACTIVE PALETTE")
                        .size(text::size::CAPTION)
                        .class(text::Variant::Decorative),
                    text(definition.metadata.name.as_str())
                        .size(34)
                        .class(text::Variant::Primary),
                    text(description).size(14).class(text::Variant::Secondary),
                    row![
                        palette_bar(seed.accent),
                        palette_bar(seed.success),
                        palette_bar(seed.warning),
                        palette_bar(seed.danger),
                        palette_bar(seed.info),
                    ]
                    .spacing(spacing::XS),
                ]
                .spacing(spacing::MD),
            )
            .class(container::Variant::Raised)
            .padding(24)
            .width(Fill),
            column![
                palette_heading("SEED COLORS", "The authored colors that shape this theme"),
                row![
                    color_swatch("Background", seed.background),
                    color_swatch("Foreground", seed.foreground),
                    color_swatch("Tint", seed.tint),
                    color_swatch("Shade", seed.shade),
                    color_swatch("Accent", seed.accent),
                    color_swatch("Success", seed.success),
                    color_swatch("Warning", seed.warning),
                    color_swatch("Danger", seed.danger),
                    color_swatch("Info", seed.info),
                ]
                .spacing(spacing::SM),
            ]
            .spacing(spacing::SM),
            row![
                column![
                    palette_heading("SEMANTIC SIGNALS", "Resolved solid fills"),
                    row![
                        semantic_palette("Accent", Intent::Accent),
                        semantic_palette("Success", Intent::Success),
                        semantic_palette("Warning", Intent::Warning),
                        semantic_palette("Danger", Intent::Danger),
                        semantic_palette("Info", Intent::Info),
                    ]
                    .spacing(spacing::SM),
                ]
                .spacing(spacing::SM)
                .width(Length::FillPortion(3)),
                column![
                    palette_heading("SURFACE SCALE", "Seven steps of depth"),
                    row![
                        surface_chip("Sunken", colors.surfaces.sunken),
                        surface_chip("Canvas", colors.surfaces.canvas),
                        surface_chip("Inset", colors.surfaces.inset),
                        surface_chip("Surface", colors.surfaces.surface),
                        surface_chip("Raised", colors.surfaces.raised),
                        surface_chip("Overlay", colors.surfaces.overlay),
                        surface_chip("Field", colors.surfaces.field),
                    ]
                    .spacing(spacing::XS),
                ]
                .spacing(spacing::SM)
                .width(Length::FillPortion(2)),
            ]
            .spacing(spacing::LG),
        ]
        .spacing(22),
    )
    .padding(28)
    .width(Fill)
    .height(Fill)
    .into()
}

fn palette_heading(title: &'static str, detail: &'static str) -> Element<'static, Message> {
    row![
        text(title).size(10).class(text::Variant::Decorative),
        rule::horizontal(1).class(rule::Variant::Subtle),
        text(detail).size(10).class(text::Variant::Muted),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn palette_bar(color: Color) -> Element<'static, Message> {
    container(Space::new())
        .style(move |theme| color_style(color, theme.definition().radius.xs))
        .height(10)
        .width(Fill)
        .into()
}

fn color_swatch(label: &'static str, color: Color) -> Element<'static, Message> {
    container(
        column![
            Space::new().height(54),
            text(label).size(10),
            text(color_hex(color)).size(9),
        ]
        .spacing(3),
    )
    .style(move |theme| color_style(color, theme.appearance().radius.md))
    .padding(10)
    .width(Fill)
    .height(112)
    .into()
}

fn semantic_palette(label: &'static str, intent: Intent) -> Element<'static, Message> {
    container(
        column![
            Space::new().height(38),
            text(label).size(text::size::LABEL),
            text("SOLID").size(9),
        ]
        .spacing(3),
    )
    .class(container::Variant::Semantic {
        intent,
        style: container::SemanticStyle::Solid,
    })
    .padding(10)
    .width(Fill)
    .height(94)
    .into()
}

fn surface_chip(label: &'static str, color: Color) -> Element<'static, Message> {
    container(column![Space::new().height(42), text(label).size(9),].spacing(3))
        .style(move |theme| {
            let mut style = color_style(color, 5.0);
            style.text_color = Some(theme.colors().content.primary.into());
            style
        })
        .padding(7)
        .width(Fill)
        .height(78)
        .into()
}

fn color_style(color: Color, radius: f32) -> container::Style {
    let iced_color: IcedColor = color.into();
    let luminance = 0.299 * iced_color.r + 0.587 * iced_color.g + 0.114 * iced_color.b;

    container::Style {
        text_color: Some(if luminance > 0.58 {
            IcedColor::from_rgb8(20, 22, 25)
        } else {
            IcedColor::WHITE
        }),
        background: Some(iced::Background::Color(iced_color)),
        border: iced::Border {
            radius: radius.into(),
            ..iced::Border::default()
        },
        ..container::Style::default()
    }
}

fn color_hex(color: Color) -> String {
    let [red, green, blue, _] = color.components();
    format!(
        "#{:02X}{:02X}{:02X}",
        (red * 255.0).round() as u8,
        (green * 255.0).round() as u8,
        (blue * 255.0).round() as u8,
    )
}

fn actions() -> Element<'static, Message> {
    container(
        column![
            section_heading(
                "BUTTON CATALOG",
                "Actions and explicit states",
                "Use the live controls, then compare the catalog output across states below.",
            ),
            live_actions(),
            matrix_heading(),
            variant_row("Primary", button::Variant::Primary),
            variant_row("Secondary", button::Variant::Secondary),
            variant_row("Outline", button::Variant::Outline),
            variant_row("Ghost", button::Variant::Ghost),
            variant_row(
                "Success solid",
                button::Variant::Semantic {
                    intent: Intent::Success,
                    style: button::SemanticStyle::Solid,
                },
            ),
            variant_row(
                "Danger soft",
                button::Variant::Semantic {
                    intent: Intent::Danger,
                    style: button::SemanticStyle::Soft,
                },
            ),
            variant_row(
                "Info outline",
                button::Variant::Semantic {
                    intent: Intent::Info,
                    style: button::SemanticStyle::Outline,
                },
            ),
        ]
        .spacing(spacing::SM),
    )
    .padding(28)
    .width(Fill)
    .height(Fill)
    .into()
}

fn live_actions() -> Element<'static, Message> {
    container(
        column![
            row![
                button(text("Create request").size(13))
                    .class(button::Variant::Primary)
                    .on_press(Message::Action),
                button(text("Save draft").size(13))
                    .class(button::Variant::Secondary)
                    .on_press(Message::Action),
                button(text("Compare").size(13))
                    .class(button::Variant::Outline)
                    .on_press(Message::Action),
                button(text("Cancel").size(13))
                    .class(button::Variant::Ghost)
                    .on_press(Message::Action),
                button(text("Disabled").size(13)).class(button::Variant::Secondary),
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center),
            row![
                button(text("Run checks").size(13))
                    .class(button::Variant::Semantic {
                        intent: Intent::Success,
                        style: button::SemanticStyle::Solid,
                    })
                    .on_press(Message::Action),
                button(text("Delete").size(13))
                    .class(button::Variant::Semantic {
                        intent: Intent::Danger,
                        style: button::SemanticStyle::Soft,
                    })
                    .on_press(Message::Action),
                button(text("Inspect info").size(13))
                    .class(button::Variant::Semantic {
                        intent: Intent::Info,
                        style: button::SemanticStyle::Outline,
                    })
                    .on_press(Message::Action),
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center),
        ]
        .spacing(spacing::SM),
    )
    .class(container::Variant::Transparent)
    .width(Fill)
    .into()
}

fn matrix_heading() -> Element<'static, Message> {
    row![
        text("VARIANT")
            .size(10)
            .class(text::Variant::Muted)
            .width(108),
        matrix_label("ACTIVE"),
        matrix_label("HOVERED"),
        matrix_label("PRESSED"),
        matrix_label("DISABLED"),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn matrix_label(label: &'static str) -> Element<'static, Message> {
    container(text(label).size(10).class(text::Variant::Muted))
        .width(Fill)
        .center_x(Length::Fill)
        .into()
}

fn variant_row(label: &'static str, variant: button::Variant) -> Element<'static, Message> {
    row![
        text(label)
            .size(text::size::LABEL)
            .class(text::Variant::Secondary)
            .width(108),
        forced_button("Active", variant, button::Status::Active),
        forced_button("Hovered", variant, button::Status::Hovered),
        forced_button("Pressed", variant, button::Status::Pressed),
        forced_button("Disabled", variant, button::Status::Disabled),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn forced_button(
    label: &'static str,
    variant: button::Variant,
    status: button::Status,
) -> Element<'static, Message> {
    button(text(label).size(text::size::LABEL))
        .style(move |theme, _runtime_status| button::appearance(theme, status, variant))
        .padding([6, 10])
        .width(Fill)
        .into()
}

fn indicators() -> Element<'static, Message> {
    container(
        column![
            section_heading(
                "CONTENT CATALOGS",
                "Separation and progress",
                "Rules organize related content; progress indicators use semantic colors only when the operation carries that meaning.",
            ),
            row![operation_progress(), quota_progress()].spacing(spacing::MD),
            progress_matrix(),
            rule_specimens(),
        ]
        .spacing(20),
    )
    .padding(28)
    .width(Fill)
    .height(Fill)
    .into()
}

fn operation_progress() -> Element<'static, Message> {
    container(
        column![
            row![
                column![
                    text("Deploying preview")
                        .size(15)
                        .class(text::Variant::Primary),
                    text("Optimizing application bundle")
                        .size(text::size::LABEL)
                        .class(text::Variant::Muted),
                ]
                .spacing(3),
                Space::new().width(Fill),
                text("68%").size(13).class(text::Variant::Secondary),
            ]
            .align_y(Alignment::Center),
            progress_bar(0.0..=100.0, 68.0)
                .class(progress_bar::Variant::Standard)
                .girth(10),
            rule::horizontal(1).class(rule::Variant::Subtle),
            text("3 of 4 checks complete")
                .size(text::size::LABEL)
                .class(text::Variant::Semantic(Intent::Success)),
        ]
        .spacing(spacing::MD),
    )
    .class(container::Variant::Raised)
    .padding(spacing::LG)
    .width(Fill)
    .into()
}

fn quota_progress() -> Element<'static, Message> {
    container(
        column![
            row![
                column![
                    text("Monthly request quota")
                        .size(15)
                        .class(text::Variant::Primary),
                    text("8,420 of 10,000 requests")
                        .size(text::size::LABEL)
                        .class(text::Variant::Muted),
                ]
                .spacing(3),
                Space::new().width(Fill),
                text("84%")
                    .size(13)
                    .class(text::Variant::Semantic(Intent::Warning)),
            ]
            .align_y(Alignment::Center),
            progress_bar(0.0..=100.0, 84.0)
                .class(progress_bar::Variant::Semantic(Intent::Warning))
                .girth(10),
            rule::horizontal(1).class(rule::Variant::Semantic(Intent::Warning)),
            text("Quota resets in 6 days")
                .size(text::size::LABEL)
                .class(text::Variant::Secondary),
        ]
        .spacing(spacing::MD),
    )
    .class(container::Variant::Raised)
    .padding(spacing::LG)
    .width(Fill)
    .into()
}

fn progress_matrix() -> Element<'static, Message> {
    container(
        column![
            row![
                text("INTENT")
                    .size(10)
                    .class(text::Variant::Muted)
                    .width(84),
                matrix_label("EMPTY"),
                matrix_label("PARTIAL"),
                matrix_label("FULL"),
            ]
            .spacing(spacing::MD)
            .align_y(Alignment::Center),
            progress_row("Accent", progress_bar::Variant::Standard, 46.0),
            progress_row(
                "Success",
                progress_bar::Variant::Semantic(Intent::Success),
                72.0,
            ),
            progress_row(
                "Warning",
                progress_bar::Variant::Semantic(Intent::Warning),
                84.0,
            ),
            progress_row(
                "Danger",
                progress_bar::Variant::Semantic(Intent::Danger),
                63.0,
            ),
            progress_row("Info", progress_bar::Variant::Semantic(Intent::Info), 35.0,),
        ]
        .spacing(spacing::MD),
    )
    .class(container::Variant::Inset)
    .padding(spacing::LG)
    .width(Fill)
    .into()
}

fn progress_row(
    label: &'static str,
    variant: progress_bar::Variant,
    partial: f32,
) -> Element<'static, Message> {
    row![
        text(label)
            .size(text::size::LABEL)
            .class(text::Variant::Secondary)
            .width(84),
        progress_cell(0.0, variant),
        progress_cell(partial, variant),
        progress_cell(100.0, variant),
    ]
    .spacing(spacing::MD)
    .align_y(Alignment::Center)
    .into()
}

fn progress_cell(value: f32, variant: progress_bar::Variant) -> Element<'static, Message> {
    progress_bar(0.0..=100.0, value)
        .class(variant)
        .girth(8)
        .into()
}

fn rule_specimens() -> Element<'static, Message> {
    row![
        container(
            column![
                text("Request summary").class(text::Variant::Primary),
                rule::horizontal(1).class(rule::Variant::Standard),
                text("POST /v1/sessions")
                    .size(text::size::LABEL)
                    .class(text::Variant::Secondary),
                rule::horizontal(2).class(rule::Variant::Strong),
                text("Validation failed")
                    .size(text::size::LABEL)
                    .class(text::Variant::Semantic(Intent::Danger)),
                rule::horizontal(2).class(rule::Variant::Semantic(Intent::Danger)),
            ]
            .spacing(10),
        )
        .class(container::Variant::Raised)
        .padding(spacing::LG)
        .width(Fill),
        container(
            row![
                column![
                    text("Environment")
                        .size(text::size::CAPTION)
                        .class(text::Variant::Muted),
                    text("Production").size(13).class(text::Variant::Primary),
                ]
                .spacing(spacing::XS)
                .width(Fill),
                rule::vertical(1).class(rule::Variant::Subtle),
                column![
                    text("Region")
                        .size(text::size::CAPTION)
                        .class(text::Variant::Muted),
                    text("us-east-1").size(13).class(text::Variant::Primary),
                ]
                .spacing(spacing::XS)
                .width(Fill),
            ]
            .spacing(spacing::LG),
        )
        .class(container::Variant::Canvas)
        .padding(spacing::LG)
        .width(Fill),
    ]
    .spacing(spacing::MD)
    .into()
}

fn selection(gallery: &Gallery) -> Element<'_, Message> {
    container(
        column![
            section_heading(
                "CONTROL CATALOGS",
                "Controls and selection",
                "Live controls exercise behavior; the compact matrices expose every style status supported by Iced.",
            ),
            row![live_selection(gallery), svg_specimens()].spacing(spacing::MD),
            tab_bar_specimens(gallery),
            row![checkbox_matrix(), radio_matrix(), toggler_matrix()].spacing(spacing::MD),
            slider_specimens(gallery),
        ]
        .spacing(18),
    )
    .padding(28)
    .width(Fill)
    .height(Fill)
    .into()
}

fn tab_bar_specimens(gallery: &Gallery) -> Element<'_, Message> {
    container(
        row![
            column![
                text("Full underline")
                    .size(text::size::CAPTION)
                    .class(text::Variant::Muted),
                tab_bar(
                    PreviewTab::ALL,
                    Some(gallery.preview_tab),
                    Message::SelectPreviewTab,
                )
                .class(tab_bar::Variant::FullUnderline),
            ]
            .spacing(spacing::SM)
            .width(Fill),
            column![
                text("Active underline")
                    .size(text::size::CAPTION)
                    .class(text::Variant::Muted),
                tab_bar(
                    PreviewTab::ALL,
                    Some(gallery.preview_tab),
                    Message::SelectPreviewTab,
                )
                .class(tab_bar::Variant::ActiveUnderline),
            ]
            .spacing(spacing::SM)
            .width(Fill),
            column![
                text("Rounded")
                    .size(text::size::CAPTION)
                    .class(text::Variant::Muted),
                tab_bar(
                    PreviewTab::ALL,
                    Some(gallery.preview_tab),
                    Message::SelectPreviewTab,
                )
                .class(tab_bar::Variant::Rounded),
            ]
            .spacing(spacing::SM)
            .width(Fill),
        ]
        .spacing(spacing::LG),
    )
    .class(container::Variant::Inset)
    .padding(spacing::LG)
    .into()
}

fn live_selection(gallery: &Gallery) -> Element<'_, Message> {
    let environments =
        Environment::ALL
            .iter()
            .fold(Column::new().spacing(spacing::SM), |col, environment| {
                col.push(
                    radio(
                        environment.label(),
                        *environment,
                        Some(gallery.environment),
                        Message::SelectEnvironment,
                    )
                    .class(radio::Variant::Standard)
                    .text_size(text::size::LABEL),
                )
            });

    container(
        row![
            column![
                text("Release options").class(text::Variant::Primary),
                checkbox(gallery.include_prereleases)
                    .label("Include prerelease builds")
                    .on_toggle(Message::IncludePrereleases)
                    .class(checkbox::Variant::Standard)
                    .text_size(text::size::LABEL),
                checkbox(gallery.accepted_terms)
                    .label("Acknowledge production impact")
                    .on_toggle(Message::AcceptTerms)
                    .class(checkbox::Variant::Invalid)
                    .text_size(text::size::LABEL),
                text("Required before production deploys")
                    .size(text::size::CAPTION)
                    .class(text::Variant::Semantic(Intent::Danger)),
            ]
            .spacing(spacing::SM)
            .width(Fill),
            column![
                text("Environment").class(text::Variant::Primary),
                environments,
            ]
            .spacing(spacing::SM)
            .width(Fill),
            column![
                text("Automation").class(text::Variant::Primary),
                toggler(gallery.auto_retry)
                    .label("Retry failed checks")
                    .on_toggle(Message::AutoRetry)
                    .class(toggler::Variant::Standard)
                    .text_size(text::size::LABEL),
                toggler(gallery.incident_alerts)
                    .label("Alert on failed deploys")
                    .on_toggle(Message::IncidentAlerts)
                    .class(toggler::Variant::Semantic(Intent::Danger))
                    .text_size(text::size::LABEL),
            ]
            .spacing(spacing::SM)
            .width(Fill),
        ]
        .spacing(spacing::LG),
    )
    .class(container::Variant::Raised)
    .padding(spacing::LG)
    .width(Length::FillPortion(3))
    .into()
}

fn svg_specimens() -> Element<'static, Message> {
    container(
        column![
            text("SVG color modes").class(text::Variant::Primary),
            row![
                live_icon("Primary", svg::Variant::Primary),
                live_icon("Muted", svg::Variant::Muted),
                live_icon("Danger", svg::Variant::Semantic(Intent::Danger)),
                live_icon("Original", svg::Variant::Original),
            ]
            .spacing(14)
            .align_y(Alignment::Center),
            rule::horizontal(1).class(rule::Variant::Subtle),
            row![
                text("FORCED")
                    .size(10)
                    .class(text::Variant::Muted)
                    .width(52),
                forced_icon("Idle", svg::Variant::Secondary, svg::Status::Idle),
                forced_icon("Hover", svg::Variant::Secondary, svg::Status::Hovered,),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(spacing::MD),
    )
    .class(container::Variant::Inset)
    .padding(spacing::LG)
    .width(Length::FillPortion(2))
    .into()
}

fn live_icon(label: &'static str, variant: svg::Variant) -> Element<'static, Message> {
    column![
        svg(svg::Handle::from_memory(COLOR_MODE_ICON))
            .class(variant)
            .width(32)
            .height(32),
        text(label).size(10).class(text::Variant::Muted),
    ]
    .spacing(5)
    .align_x(Alignment::Center)
    .into()
}

fn forced_icon(
    label: &'static str,
    variant: svg::Variant,
    status: svg::Status,
) -> Element<'static, Message> {
    row![
        svg(svg::Handle::from_memory(COLOR_MODE_ICON))
            .style(move |theme, _| svg::appearance(theme, status, variant))
            .width(20)
            .height(20),
        text(label).size(10).class(text::Variant::Secondary),
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .into()
}

fn checkbox_matrix() -> Element<'static, Message> {
    state_matrix(
        "Checkbox",
        column![
            state_header("STATE", "OFF", "ON"),
            checkbox_state_row("Active", checkbox::Status::Active { is_checked: false }),
            checkbox_state_row("Hovered", checkbox::Status::Hovered { is_checked: false }),
            checkbox_state_row("Disabled", checkbox::Status::Disabled { is_checked: false }),
        ]
        .spacing(spacing::SM),
    )
}

fn checkbox_state_row(label: &'static str, status: checkbox::Status) -> Element<'static, Message> {
    let checked_status = match status {
        checkbox::Status::Active { .. } => checkbox::Status::Active { is_checked: true },
        checkbox::Status::Hovered { .. } => checkbox::Status::Hovered { is_checked: true },
        checkbox::Status::Disabled { .. } => checkbox::Status::Disabled { is_checked: true },
    };

    row![
        text(label).size(10).class(text::Variant::Muted).width(58),
        state_cell(forced_checkbox(false, status)),
        state_cell(forced_checkbox(true, checked_status)),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn forced_checkbox(is_checked: bool, status: checkbox::Status) -> Element<'static, Message> {
    checkbox(is_checked)
        .style(move |theme, _| checkbox::appearance(theme, status, checkbox::Variant::Standard))
        .size(16)
        .into()
}

fn radio_matrix() -> Element<'static, Message> {
    state_matrix(
        "Radio",
        column![
            state_header("STATE", "OFF", "ON"),
            radio_state_row("Active", radio::Status::Active { is_selected: false }),
            radio_state_row("Hovered", radio::Status::Hovered { is_selected: false }),
            text("No disabled or focus status")
                .size(10)
                .class(text::Variant::Muted),
        ]
        .spacing(spacing::SM),
    )
}

fn radio_state_row(label: &'static str, status: radio::Status) -> Element<'static, Message> {
    let selected_status = match status {
        radio::Status::Active { .. } => radio::Status::Active { is_selected: true },
        radio::Status::Hovered { .. } => radio::Status::Hovered { is_selected: true },
    };

    row![
        text(label).size(10).class(text::Variant::Muted).width(58),
        state_cell(forced_radio(false, status)),
        state_cell(forced_radio(true, selected_status)),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn forced_radio(is_selected: bool, status: radio::Status) -> Element<'static, Message> {
    radio("", true, is_selected.then_some(true), |_| Message::Action)
        .style(move |theme, _| radio::appearance(theme, status, radio::Variant::Standard))
        .size(16)
        .into()
}

fn toggler_matrix() -> Element<'static, Message> {
    state_matrix(
        "Toggler",
        column![
            state_header("STATE", "OFF", "ON"),
            toggler_state_row("Active", toggler::Status::Active { is_toggled: false }),
            toggler_state_row("Hovered", toggler::Status::Hovered { is_toggled: false }),
            toggler_state_row("Disabled", toggler::Status::Disabled { is_toggled: false }),
            rule::horizontal(1).class(rule::Variant::Subtle),
            text("SIZES").size(9).class(text::Variant::Muted),
            toggler_size_row("Compact", toggler::size::COMPACT),
            toggler_size_row("Default", toggler::size::DEFAULT),
        ]
        .spacing(spacing::SM),
    )
}

fn toggler_size_row(label: &'static str, size: f32) -> Element<'static, Message> {
    row![
        text(label).size(10).class(text::Variant::Muted).width(58),
        toggler(true).on_toggle(|_| Message::Action).size(size),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn toggler_state_row(label: &'static str, status: toggler::Status) -> Element<'static, Message> {
    let toggled_status = match status {
        toggler::Status::Active { .. } => toggler::Status::Active { is_toggled: true },
        toggler::Status::Hovered { .. } => toggler::Status::Hovered { is_toggled: true },
        toggler::Status::Disabled { .. } => toggler::Status::Disabled { is_toggled: true },
    };

    row![
        text(label).size(10).class(text::Variant::Muted).width(58),
        state_cell(forced_toggler(false, status)),
        state_cell(forced_toggler(true, toggled_status)),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn forced_toggler(is_toggled: bool, status: toggler::Status) -> Element<'static, Message> {
    toggler(is_toggled)
        .style(move |theme, _| toggler::appearance(theme, status, toggler::Variant::Standard))
        .into()
}

fn state_header(
    state: &'static str,
    off: &'static str,
    on: &'static str,
) -> Element<'static, Message> {
    row![
        text(state).size(9).class(text::Variant::Muted).width(58),
        state_cell(text(off).size(9).class(text::Variant::Muted)),
        state_cell(text(on).size(9).class(text::Variant::Muted)),
    ]
    .spacing(spacing::SM)
    .into()
}

fn state_cell(content: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    container(content).width(Fill).center_x(Length::Fill).into()
}

fn state_matrix(
    title: &'static str,
    content: impl Into<Element<'static, Message>>,
) -> Element<'static, Message> {
    container(
        column![
            text(title).size(13).class(text::Variant::Primary),
            content.into()
        ]
        .spacing(10),
    )
    .class(container::Variant::Inset)
    .padding(14)
    .width(Fill)
    .into()
}

fn slider_specimens(gallery: &Gallery) -> Element<'_, Message> {
    container(
        row![
            column![
                text("Live range controls")
                    .size(13)
                    .class(text::Variant::Primary),
                slider_control(
                    "Canvas zoom",
                    50.0..=200.0,
                    gallery.zoom,
                    "%",
                    slider::Variant::Standard,
                    Message::ZoomChanged,
                ),
                slider_control(
                    "Alert threshold",
                    0.0..=100.0,
                    gallery.alert_threshold,
                    "%",
                    slider::Variant::Semantic(Intent::Warning),
                    Message::AlertThresholdChanged,
                ),
            ]
            .spacing(spacing::MD)
            .width(Length::FillPortion(2)),
            rule::vertical(1).class(rule::Variant::Subtle),
            column![
                text("Explicit states")
                    .size(13)
                    .class(text::Variant::Primary),
                slider_header(),
                slider_state_row("Active", slider::Status::Active),
                slider_state_row("Hovered", slider::Status::Hovered),
                slider_state_row("Dragged", slider::Status::Dragged),
            ]
            .spacing(spacing::SM)
            .width(Length::FillPortion(3)),
        ]
        .spacing(spacing::LG),
    )
    .class(container::Variant::Raised)
    .padding(spacing::LG)
    .width(Fill)
    .into()
}

fn slider_control<'a>(
    label: &'static str,
    range: std::ops::RangeInclusive<f32>,
    value: f32,
    suffix: &'static str,
    variant: slider::Variant,
    on_change: impl Fn(f32) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        row![
            text(label)
                .size(text::size::CAPTION)
                .class(text::Variant::Secondary),
            Space::new().width(Fill),
            text(format!("{value:.0}{suffix}"))
                .size(text::size::CAPTION)
                .class(text::Variant::Muted),
        ]
        .align_y(Alignment::Center),
        slider(range, value, on_change).class(variant).step(1.0),
    ]
    .spacing(5)
    .into()
}

fn slider_header() -> Element<'static, Message> {
    row![
        text("STATE").size(9).class(text::Variant::Muted).width(58),
        state_cell(text("MIN").size(9).class(text::Variant::Muted)),
        state_cell(text("MID").size(9).class(text::Variant::Muted)),
        state_cell(text("MAX").size(9).class(text::Variant::Muted)),
    ]
    .spacing(spacing::SM)
    .into()
}

fn slider_state_row(label: &'static str, status: slider::Status) -> Element<'static, Message> {
    row![
        text(label).size(10).class(text::Variant::Muted).width(58),
        forced_slider(0.0, status),
        forced_slider(50.0, status),
        forced_slider(100.0, status),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn forced_slider(value: f32, status: slider::Status) -> Element<'static, Message> {
    slider(0.0..=100.0, value, |_| Message::Action)
        .style(move |theme, _| slider::appearance(theme, status, slider::Variant::Standard))
        .width(Fill)
        .into()
}

fn fields(gallery: &Gallery) -> Element<'_, Message> {
    container(
        column![
            section_heading(
                "FIELD CATALOGS",
                "Structured input",
                "Live values exercise editing while the matrix fixes equivalent single-line and multiline controls in every exposed state.",
            ),
            row![live_fields(gallery), live_editor(gallery)].spacing(spacing::MD),
            field_state_matrix(gallery),
        ]
        .spacing(18),
    )
    .padding(28)
    .width(Fill)
    .height(Fill)
    .into()
}

fn live_fields(gallery: &Gallery) -> Element<'_, Message> {
    container(
        column![
            text("Request details").class(text::Variant::Primary),
            labeled_input(
                "Request name",
                "Name this request",
                &gallery.request_name,
                Field::RequestName,
                text_input::Variant::Standard,
            )
            .required(true)
            .trailing(
                text(format!("{} / 60", gallery.request_name.chars().count()))
                    .size(text::size::CAPTION)
                    .class(text::Variant::Muted),
            ),
            labeled_input(
                "Search",
                "Filter headers and variables",
                &gallery.search_query,
                Field::Search,
                text_input::Variant::Subtle,
            ),
            labeled_input(
                "Callback URL",
                "https://example.com/hook",
                &gallery.endpoint,
                Field::Endpoint,
                text_input::Variant::Invalid,
            )
            .required(true)
            .error("Enter an HTTPS callback URL"),
            labeled_input(
                "Environment value",
                "KEY=value",
                &gallery.code_value,
                Field::CodeValue,
                text_input::Variant::Code,
            ),
        ]
        .spacing(spacing::SM),
    )
    .class(container::Variant::Raised)
    .padding(spacing::LG)
    .width(Length::FillPortion(3))
    .into()
}

fn labeled_input<'a>(
    label: &'static str,
    placeholder: &'static str,
    value: &'a str,
    field: Field,
    variant: text_input::Variant,
) -> Labeled<'a, Message> {
    labeled(
        label,
        text_input(placeholder, value)
            .on_input(move |value| Message::EditField(field, value))
            .class(variant)
            .padding([spacing::SM, 10.0]),
    )
}

fn live_editor(gallery: &Gallery) -> Element<'_, Message> {
    container(
        column![
            text("Deployment notes").class(text::Variant::Primary),
            text("Plain text editor with code-oriented field treatment")
                .size(text::size::CAPTION)
                .class(text::Variant::Muted),
            text_editor(&gallery.notes)
                .placeholder("Add rollout notes...")
                .on_action(Message::EditNotes)
                .class(text_editor::Variant::Code)
                .font(iced::Font::MONOSPACE)
                .height(172)
                .padding(spacing::MD),
            text(format!("{} lines", gallery.notes.line_count()))
                .size(10)
                .class(text::Variant::Muted),
        ]
        .spacing(9),
    )
    .class(container::Variant::Inset)
    .padding(spacing::LG)
    .width(Length::FillPortion(2))
    .into()
}

fn field_state_matrix(gallery: &Gallery) -> Element<'_, Message> {
    container(
        column![
            row![
                text("STATE").size(9).class(text::Variant::Muted).width(96),
                matrix_label("TEXT INPUT"),
                matrix_label("TEXT EDITOR"),
            ]
            .spacing(10),
            field_state_row(gallery, "Active", text_input::Status::Active),
            field_state_row(gallery, "Hovered", text_input::Status::Hovered),
            field_state_row(
                gallery,
                "Focused",
                text_input::Status::Focused { is_hovered: false },
            ),
            field_state_row(
                gallery,
                "Focus + hover",
                text_input::Status::Focused { is_hovered: true },
            ),
            field_state_row(gallery, "Disabled", text_input::Status::Disabled),
        ]
        .spacing(spacing::SM),
    )
    .class(container::Variant::Inset)
    .padding(14)
    .width(Fill)
    .into()
}

fn field_state_row<'a>(
    gallery: &'a Gallery,
    label: &'static str,
    status: text_input::Status,
) -> Element<'a, Message> {
    let editor_status = match status {
        text_input::Status::Active => text_editor::Status::Active,
        text_input::Status::Hovered => text_editor::Status::Hovered,
        text_input::Status::Focused { is_hovered } => text_editor::Status::Focused { is_hovered },
        text_input::Status::Disabled => text_editor::Status::Disabled,
    };

    row![
        text(label).size(10).class(text::Variant::Muted).width(96),
        text_input("Placeholder", "Example value")
            .on_input(|_| Message::Action)
            .style(move |theme, _| {
                text_input::appearance(theme, status, text_input::Variant::Standard)
            })
            .padding([6, 9])
            .width(Fill),
        text_editor(&gallery.notes)
            .on_action(Message::EditNotes)
            .style(move |theme, _| {
                text_editor::appearance(theme, editor_status, text_editor::Variant::Standard)
            })
            .height(34)
            .padding([6, 9]),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

fn choice(gallery: &Gallery) -> Element<'_, Message> {
    container(
        column![
            section_heading(
                "CHOICE & NAVIGATION",
                "Menus and bounded regions",
                "Open the environment field to inspect the menu overlay, then compare real overflow behavior across each supported scroll direction.",
            ),
            row![pick_list_specimens(gallery), pick_list_matrix()].spacing(spacing::MD),
            scrollable_specimens(),
            scrollable_state_matrix(),
        ]
        .spacing(spacing::LG),
    )
    .padding(28)
    .width(Fill)
    .height(Fill)
    .into()
}

fn pick_list_specimens(gallery: &Gallery) -> Element<'_, Message> {
    container(
        column![
            text("Live environment selector").class(text::Variant::Primary),
            pick_list(Some(gallery.environment), Environment::ALL, |environment| {
                environment.label().to_owned()
            },)
            .on_select(Message::SelectEnvironment)
            .class(pick_list::Variant::Standard)
            .width(Fill),
            pick_list(
                Option::<Environment>::None,
                Environment::ALL,
                |environment| environment.label().to_owned(),
            )
            .placeholder("Subtle placeholder")
            .on_select(Message::SelectEnvironment)
            .class(pick_list::Variant::Subtle)
            .width(Fill),
            pick_list(
                Option::<Environment>::None,
                Environment::ALL,
                |environment| environment.label().to_owned(),
            )
            .placeholder("Selection required")
            .on_select(Message::SelectEnvironment)
            .class(pick_list::Variant::Invalid)
            .width(Fill),
        ]
        .spacing(9),
    )
    .class(container::Variant::Raised)
    .padding(spacing::LG)
    .width(Length::FillPortion(2))
    .into()
}

fn pick_list_matrix() -> Element<'static, Message> {
    container(
        column![
            text("Explicit states")
                .size(13)
                .class(text::Variant::Primary),
            pick_list_state("Active", pick_list::Status::Active),
            pick_list_state("Hovered", pick_list::Status::Hovered),
            pick_list_state("Opened", pick_list::Status::Opened { is_hovered: false },),
            pick_list_state(
                "Open + hover",
                pick_list::Status::Opened { is_hovered: true },
            ),
            pick_list_state("Disabled", pick_list::Status::Disabled),
        ]
        .spacing(spacing::SM),
    )
    .class(container::Variant::Inset)
    .padding(14)
    .width(Length::FillPortion(3))
    .into()
}

fn pick_list_state(label: &'static str, status: pick_list::Status) -> Element<'static, Message> {
    row![
        text(label).size(10).class(text::Variant::Muted).width(82),
        pick_list(
            Some(Environment::Staging),
            Environment::ALL,
            |environment| environment.label().to_owned(),
        )
        .on_select(Message::SelectEnvironment)
        .style(move |theme, _| {
            pick_list::appearance(theme, status, pick_list::Variant::Standard)
        })
        .width(Fill),
    ]
    .spacing(spacing::SM)
    .align_y(Alignment::Center)
    .into()
}

fn scrollable_specimens() -> Element<'static, Message> {
    row![
        scroll_panel(
            "Vertical",
            scrollable(
                column![
                    log_line("10:42:01", "Queued build"),
                    log_line("10:42:03", "Resolved dependencies"),
                    log_line("10:42:08", "Compiled workspace"),
                    log_line("10:42:10", "Ran policy checks"),
                    log_line("10:42:12", "Uploaded artifact"),
                    log_line("10:42:14", "Started rollout"),
                ]
                .spacing(7),
            )
            .class(scrollable::Variant::Standard)
            .height(104)
            .width(Fill),
        ),
        scroll_panel(
            "Horizontal",
            scrollable(
                row![
                    metric("REQUESTS", "1,284"),
                    metric("ERRORS", "12"),
                    metric("LATENCY", "86 ms"),
                    metric("REGIONS", "4"),
                ]
                .spacing(spacing::SM),
            )
            .direction(scrollable::Direction::Horizontal(scrollable::scrollbar()))
            .class(scrollable::Variant::Subtle)
            .height(104)
            .width(Fill),
        ),
        scroll_panel(
            "Both axes",
            scrollable(
                container(
                    column![
                        text("Deployment topology")
                            .size(text::size::LABEL)
                            .class(text::Variant::Primary),
                        row![
                            metric("US EAST", "Healthy"),
                            metric("US WEST", "Healthy"),
                            metric("EU WEST", "Rolling"),
                            metric("AP SOUTH", "Queued"),
                        ]
                        .spacing(spacing::SM),
                    ]
                    .spacing(14),
                )
                .class(container::Variant::Canvas)
                .padding(14)
                .width(580)
                .height(170),
            )
            .direction(scrollable::Direction::Both {
                vertical: scrollable::scrollbar(),
                horizontal: scrollable::scrollbar(),
            })
            .class(scrollable::Variant::Standard)
            .height(104)
            .width(Fill),
        ),
    ]
    .spacing(spacing::MD)
    .into()
}

fn scroll_panel(
    title: &'static str,
    content: scrollable::Scrollable<'static, Message>,
) -> Element<'static, Message> {
    container(
        column![
            text(title)
                .size(text::size::CAPTION)
                .class(text::Variant::Muted),
            content,
        ]
        .spacing(spacing::SM),
    )
    .class(container::Variant::Inset)
    .padding(spacing::MD)
    .width(Fill)
    .into()
}

fn log_line(time: &'static str, event: &'static str) -> Element<'static, Message> {
    row![
        text(time).size(10).class(text::Variant::Muted).width(58),
        text(event).size(10).class(text::Variant::Secondary),
    ]
    .spacing(6)
    .into()
}

fn metric(label: &'static str, value: &'static str) -> Element<'static, Message> {
    container(
        column![
            text(label).size(9).class(text::Variant::Muted),
            text(value)
                .size(text::size::LABEL)
                .class(text::Variant::Primary),
        ]
        .spacing(spacing::XS),
    )
    .class(container::Variant::Raised)
    .padding(10)
    .width(132)
    .into()
}

fn scrollable_state_matrix() -> Element<'static, Message> {
    container(
        column![
            text("Forced rail states")
                .size(13)
                .class(text::Variant::Primary),
            row![
                forced_scrollable(
                    "Active",
                    scrollable::Status::Active {
                        is_horizontal_scrollbar_disabled: true,
                        is_vertical_scrollbar_disabled: false,
                    },
                ),
                forced_scrollable(
                    "Hovered",
                    scrollable::Status::Hovered {
                        is_horizontal_scrollbar_hovered: false,
                        is_vertical_scrollbar_hovered: true,
                        is_horizontal_scrollbar_disabled: true,
                        is_vertical_scrollbar_disabled: false,
                    },
                ),
                forced_scrollable(
                    "Dragged",
                    scrollable::Status::Dragged {
                        is_horizontal_scrollbar_dragged: false,
                        is_vertical_scrollbar_dragged: true,
                        is_horizontal_scrollbar_disabled: true,
                        is_vertical_scrollbar_disabled: false,
                    },
                ),
            ]
            .spacing(10),
        ]
        .spacing(spacing::SM),
    )
    .class(container::Variant::Raised)
    .padding(spacing::MD)
    .width(Fill)
    .into()
}

fn forced_scrollable(label: &'static str, status: scrollable::Status) -> Element<'static, Message> {
    column![
        text(label).size(9).class(text::Variant::Muted),
        scrollable(
            column![
                text("Worker ready").size(10),
                text("Build complete").size(10),
                text("Checks passed").size(10),
                text("Artifact uploaded").size(10),
            ]
            .spacing(6),
        )
        .style(move |theme, _| {
            scrollable::appearance(theme, status, scrollable::Variant::Standard)
        })
        .height(52)
        .width(Fill),
    ]
    .spacing(5)
    .width(Fill)
    .into()
}

fn surfaces() -> Element<'static, Message> {
    container(
        column![
            section_heading(
                "CONTENT FOUNDATIONS",
                "Planes and readable roles",
                "Depth comes from ordered surfaces and boundaries. Shadows are reserved for floating placement.",
            ),
            row![
                surface_swatch("Sunken", "Code wells", container::Variant::Sunken),
                surface_swatch("Canvas", "App chrome", container::Variant::Canvas),
                surface_swatch("Inset", "Local recess", container::Variant::Inset),
                surface_swatch("Surface", "Primary plane", container::Variant::Surface),
            ]
            .spacing(10),
            row![
                surface_swatch("Raised", "Inline panel", container::Variant::Raised),
                surface_swatch("Overlay", "Placed above", container::Variant::Overlay),
                surface_swatch("Field", "Editable plane", container::Variant::Field),
                surface_swatch("Popover", "Floating", container::Variant::Popover),
            ]
            .spacing(10),
            container(
                row![
                    content_sample("Primary", text::Variant::Primary),
                    content_sample("Secondary", text::Variant::Secondary),
                    content_sample("Muted", text::Variant::Muted),
                    content_sample("Decorative", text::Variant::Decorative),
                    content_sample("Disabled", text::Variant::Disabled),
                ]
                .spacing(spacing::LG),
            )
            .class(container::Variant::Inset)
            .padding(spacing::LG)
            .width(Fill),
            container(
                row![
                    type_scale_sample("Caption", text::size::CAPTION),
                    type_scale_sample("Label", text::size::LABEL),
                    type_scale_sample("Body", text::size::BODY),
                    type_scale_sample("Title", text::size::TITLE),
                    type_scale_sample("Heading", text::size::HEADING),
                    type_scale_sample("Display", text::size::DISPLAY),
                ]
                .spacing(spacing::LG)
                .align_y(Alignment::End),
            )
            .class(container::Variant::Raised)
            .padding(spacing::LG)
            .width(Fill),
            row![
                semantic_sample("Success", Intent::Success),
                semantic_sample("Warning", Intent::Warning),
                semantic_sample("Danger", Intent::Danger),
                semantic_sample("Info", Intent::Info),
            ]
            .spacing(10),
        ]
        .spacing(18),
    )
    .padding(28)
    .width(Fill)
    .height(Fill)
    .into()
}

fn surface_swatch(
    title: &'static str,
    detail: &'static str,
    variant: container::Variant,
) -> Element<'static, Message> {
    container(
        column![
            text(title).class(text::Variant::Primary),
            text(detail)
                .size(text::size::CAPTION)
                .class(text::Variant::Muted),
        ]
        .spacing(6),
    )
    .class(variant)
    .padding(14)
    .width(Fill)
    .height(92)
    .into()
}

fn content_sample(label: &'static str, variant: text::Variant) -> Element<'static, Message> {
    column![
        text("Aa").size(text::size::DISPLAY).class(variant),
        text(label).size(text::size::LABEL).class(variant),
    ]
    .spacing(3)
    .width(Fill)
    .into()
}

fn type_scale_sample(label: &'static str, size: f32) -> Element<'static, Message> {
    column![
        text("Aa").size(size).class(text::Variant::Primary),
        text(label)
            .size(text::size::CAPTION)
            .class(text::Variant::Muted),
    ]
    .spacing(spacing::XS)
    .width(Fill)
    .into()
}

fn semantic_sample(label: &'static str, intent: Intent) -> Element<'static, Message> {
    container(column![text(label), text("Soft pair").size(text::size::LABEL),].spacing(spacing::XS))
        .class(container::Variant::Semantic {
            intent,
            style: container::SemanticStyle::Soft,
        })
        .padding(14)
        .width(Fill)
        .into()
}

fn overlays(gallery: &Gallery) -> Element<'_, Message> {
    container(
        column![
            section_heading(
                "OVERLAY CATALOGS",
                "Deployment controls",
                "A focused release workspace for transient feedback, action menus, and modal confirmation.",
            ),
            deployment_summary(gallery),
            row![
                feedback_panel(),
                action_menu_panel(),
            ]
            .spacing(spacing::MD),
            tooltip_panel(),
        ]
        .spacing(18),
    )
    .padding(28)
    .width(Fill)
    .height(Fill)
    .into()
}

fn deployment_summary(gallery: &Gallery) -> Element<'_, Message> {
    container(
        row![
            container(text("PROD").size(10))
                .class(container::Variant::Semantic {
                    intent: Intent::Success,
                    style: container::SemanticStyle::Soft,
                })
                .padding([6, 9]),
            column![
                text("Production deployment")
                    .size(text::size::TITLE)
                    .class(text::Variant::Primary),
                text("main  •  us-east-1  •  rolling")
                    .size(text::size::CAPTION)
                    .class(text::Variant::Muted),
            ]
            .spacing(4),
            Space::new().width(Fill),
            column![
                text(if gallery.dialog.is_visible() {
                    "AWAITING CONFIRMATION"
                } else {
                    "READY TO DEPLOY"
                })
                .size(9)
                .class(text::Variant::Decorative),
                text("Theme preview ready")
                    .size(text::size::CAPTION)
                    .class(text::Variant::Secondary),
            ]
            .spacing(4)
            .align_x(Alignment::End),
            button(text("Review and deploy").size(text::size::LABEL))
                .class(button::Variant::Primary)
                .padding([9, 14])
                .on_press(Message::OpenDialog),
        ]
        .spacing(spacing::MD)
        .align_y(Alignment::Center),
    )
    .class(container::Variant::Raised)
    .padding(spacing::LG)
    .width(Fill)
    .into()
}

fn feedback_panel() -> Element<'static, Message> {
    container(
        column![
            text("01  TRANSIENT FEEDBACK")
                .size(10)
                .class(text::Variant::Decorative),
            text("Notify the team")
                .size(15)
                .class(text::Variant::Primary),
            text("Preview each intent as a live snackbar notification.")
                .size(text::size::CAPTION)
                .class(text::Variant::Muted),
            rule::horizontal(1).class(rule::Variant::Subtle),
            column![
                row![
                    snackbar_button("Primary", snackbar::Variant::Primary),
                    snackbar_button("Secondary", snackbar::Variant::Secondary),
                    snackbar_button("Success", snackbar::Variant::Semantic(Intent::Success),),
                ]
                .spacing(spacing::SM),
                row![
                    snackbar_button("Warning", snackbar::Variant::Semantic(Intent::Warning),),
                    snackbar_button("Danger", snackbar::Variant::Semantic(Intent::Danger)),
                    snackbar_button("Info", snackbar::Variant::Semantic(Intent::Info)),
                ]
                .spacing(spacing::SM),
            ]
            .spacing(spacing::SM),
        ]
        .spacing(spacing::SM),
    )
    .class(container::Variant::Raised)
    .padding(spacing::LG)
    .width(Fill)
    .into()
}

fn snackbar_button(label: &str, variant: snackbar::Variant) -> Element<'_, Message> {
    let button_variant = match variant {
        snackbar::Variant::Primary => button::Variant::Primary,
        snackbar::Variant::Secondary => button::Variant::Secondary,
        snackbar::Variant::Semantic(intent) => button::Variant::Semantic {
            intent,
            style: button::SemanticStyle::Soft,
        },
        _ => button::Variant::Secondary,
    };

    button(text(label).size(text::size::CAPTION))
        .class(button_variant)
        .padding([7, 10])
        .width(Fill)
        .on_press(Message::ShowSnackbar(variant))
        .into()
}

fn action_menu_panel() -> Element<'static, Message> {
    container(
        column![
            text("02  CONTEXTUAL ACTIONS")
                .size(10)
                .class(text::Variant::Decorative),
            text("Release commands")
                .size(15)
                .class(text::Variant::Primary),
            text("Open either menu to inspect hierarchy and placement.")
                .size(text::size::CAPTION)
                .class(text::Variant::Muted),
            rule::horizontal(1).class(rule::Variant::Subtle),
            row![
                dropdown(
                    text("Release menu  ↓"),
                    [
                        dropdown::section("Release"),
                        dropdown::item("Deploy preview", Message::Action,),
                        dropdown::item("Promote to staging", Message::Action,),
                        dropdown::separator(),
                        dropdown::item("Rollback release", Message::Action),
                    ],
                )
                .width(158),
                dropdown(
                    text("Configuration  ↓"),
                    [
                        dropdown::section("Environment"),
                        dropdown::item("Edit variables", Message::Action,),
                        dropdown::item("View audit trail", Message::Action,),
                    ],
                )
                .class(dropdown::Variant::Subtle)
                .menu_class(dropdown::menu::Variant::Subtle)
                .width(158),
            ]
            .spacing(spacing::SM),
            context_menu(
                container(
                    text("Right-click for artifact actions")
                        .size(text::size::CAPTION)
                        .class(text::Variant::Muted),
                )
                .class(container::Variant::Raised)
                .padding([10, 12])
                .width(Fill),
                [
                    context_menu::item("Open build log", Message::Action).shortcut(
                        context_menu::Shortcut::command(context_menu::Key::Character('l')),
                    ),
                    context_menu::item("Copy artifact URL", Message::Action).shortcut(
                        context_menu::Shortcut::shift_command(context_menu::Key::Character('c')),
                    ),
                    context_menu::separator(),
                    context_menu::submenu(
                        "Promote to",
                        [
                            context_menu::item("Staging", Message::Action),
                            context_menu::item("Production", Message::Action),
                            context_menu::item("Canary", Message::Action).disabled(true),
                        ],
                    ),
                    context_menu::submenu(
                        "Deploy to region",
                        DEPLOY_REGIONS
                            .iter()
                            .map(|region| context_menu::item(*region, Message::Action)),
                    ),
                    context_menu::separator(),
                    context_menu::item_maybe("Delete artifact", None),
                ],
            ),
        ]
        .spacing(spacing::SM),
    )
    .class(container::Variant::Inset)
    .padding(spacing::LG)
    .width(Fill)
    .into()
}

// Long enough to exceed the context menu's max height so the submenu scrolls.
const DEPLOY_REGIONS: &[&str] = &[
    "us-east-1",
    "us-east-2",
    "us-west-1",
    "us-west-2",
    "ca-central-1",
    "sa-east-1",
    "eu-west-1",
    "eu-west-2",
    "eu-west-3",
    "eu-central-1",
    "eu-north-1",
    "eu-south-1",
    "me-south-1",
    "af-south-1",
    "ap-south-1",
    "ap-east-1",
    "ap-northeast-1",
    "ap-northeast-2",
    "ap-southeast-1",
    "ap-southeast-2",
];

fn tooltip_panel() -> Element<'static, Message> {
    container(
        column![
            text("03  TOOLTIP PLACEMENT")
                .size(10)
                .class(text::Variant::Decorative),
            text("Context at the point of action")
                .size(15)
                .class(text::Variant::Primary),
            text("Hover each control to compare placement and viewport behavior.")
                .size(text::size::CAPTION)
                .class(text::Variant::Muted),
            rule::horizontal(1).class(rule::Variant::Subtle),
            row![
                tooltip_button("Above", "Appears above the anchor", tooltip::Position::Top),
                tooltip_button(
                    "Below",
                    "Appears below the anchor",
                    tooltip::Position::Bottom,
                ),
                tooltip_button(
                    "Left",
                    "Appears left of the anchor",
                    tooltip::Position::Left
                ),
                tooltip_button(
                    "Right",
                    "Appears right of the anchor",
                    tooltip::Position::Right,
                ),
                tooltip_button(
                    "Follow",
                    "Tracks the pointer",
                    tooltip::Position::FollowCursor,
                ),
                tooltip(
                    button(
                        svg(svg::Handle::from_memory(COLOR_MODE_ICON))
                            .class(svg::Variant::Original)
                            .width(18)
                            .height(18),
                    )
                    .class(button::Variant::Icon)
                    .padding(6)
                    .on_press(Message::Action),
                    text("Immediate icon tooltip").size(text::size::CAPTION),
                    tooltip::Position::Bottom,
                )
                .delay(iced::time::Duration::ZERO),
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center),
        ]
        .spacing(spacing::SM),
    )
    .class(container::Variant::Raised)
    .padding(spacing::LG)
    .width(Fill)
    .into()
}

fn tooltip_button(
    label: &'static str,
    hint: &'static str,
    position: tooltip::Position,
) -> Element<'static, Message> {
    tooltip(
        button(text(label))
            .class(button::Variant::Outline)
            .padding([7, 10])
            .on_press(Message::Action),
        text(hint).size(text::size::LABEL),
        position,
    )
    .into()
}

fn section_heading(
    eyebrow: &'static str,
    title: &'static str,
    detail: &'static str,
) -> Element<'static, Message> {
    column![
        text(eyebrow)
            .size(text::size::CAPTION)
            .class(text::Variant::Decorative),
        text(title).size(26).class(text::Variant::Primary),
        text(detail).size(13).class(text::Variant::Secondary),
    ]
    .spacing(5)
    .into()
}
