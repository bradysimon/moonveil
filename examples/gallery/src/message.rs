use crate::section::Section;
use crate::shared::{Environment, Field, PreviewTab, Rounding, ThemeChoice};
use moonveil::widget::{dialog, markdown, snackbar, text_editor};

#[derive(Debug, Clone)]
pub(super) enum Message {
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
    EditMarkdown(text_editor::Action),
    OpenLink(markdown::Uri),
}
