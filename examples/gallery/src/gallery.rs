use crate::shared::{Environment, Field, PreviewTab, Rounding, ThemeChoice};
use crate::ui;
use crate::message::Message;
use crate::section::Section;
use crate::sections;
use iced::Length::Fill;
use iced::widget::row;
use moonveil::widget::text::{self, TextExt as _, text};
use moonveil::widget::{
    button, combo_box, container, dialog, markdown, rule, snackbar, text_editor,
};
use moonveil::{Element, Theme};

pub(super) struct Gallery {
    pub(super) theme: Theme,
    pub(super) theme_choice: ThemeChoice,
    pub(super) rounding: Rounding,
    pub(super) section: Section,
    pub(super) include_prereleases: bool,
    pub(super) accepted_terms: bool,
    pub(super) environment: Environment,
    pub(super) environments: combo_box::State<Environment>,
    pub(super) preview_tab: PreviewTab,
    pub(super) auto_retry: bool,
    pub(super) incident_alerts: bool,
    pub(super) zoom: f32,
    pub(super) alert_threshold: f32,
    pub(super) request_name: String,
    pub(super) search_query: String,
    pub(super) endpoint: String,
    pub(super) code_value: String,
    pub(super) notes: text_editor::Content,
    pub(super) markdown_source: text_editor::Content,
    pub(super) markdown: markdown::Content,
    pub(super) snackbars: snackbar::State<Message>,
    pub(super) dialog: dialog::State,
}

impl Gallery {
    pub fn new() -> Self {
        Gallery {
            theme: ThemeChoice::Dark.resolve(Rounding::Standard),
            theme_choice: ThemeChoice::Dark,
            rounding: Rounding::Standard,
            section: Section::Overview,
            include_prereleases: true,
            accepted_terms: false,
            environment: Environment::Staging,
            environments: combo_box::State::new(Environment::ALL.to_vec()),
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
            markdown_source: text_editor::Content::with_text(sections::markdown::MARKDOWN_SAMPLE),
            markdown: markdown::Content::parse(sections::markdown::MARKDOWN_SAMPLE),
            snackbars: snackbar::State::default(),
            dialog: dialog::State::default(),
        }
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn update(gallery: &mut Gallery, message: Message) {
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
            Message::EditMarkdown(action) => {
                let is_edit = action.is_edit();
                gallery.markdown_source.perform(action);

                if is_edit {
                    gallery.markdown = markdown::Content::parse(&gallery.markdown_source.text());
                }
            }
            Message::OpenLink(url) => {
                gallery
                    .snackbars
                    .push(snackbar::Entry::new("Link clicked").description(url));
            }
        }
    }

    pub fn view(gallery: &Gallery) -> Element<'_, Message> {
        let base = container(row![
            ui::sidebar(gallery),
            rule::vertical(1).class(rule::Variant::Subtle),
            ui::workspace(gallery)
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

        dialog(
            snackbar(&gallery.snackbars, base).on_update(Message::Snackbar),
            &gallery.dialog,
            dialog_config,
        )
        .on_update(Message::Dialog)
        .into()
    }
}
