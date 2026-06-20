use super::tokens::{
    DIALOG_ERROR, DIALOG_ERROR_BORDER, DIALOG_INFO, DIALOG_INFO_BORDER, DIALOG_WARNING,
    DIALOG_WARNING_BORDER,
};
use super::variants::variant_contains_any;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum DialogSeverity {
    Info,
    Warning,
    Error,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn severity(
    node: &TemplatePaneNodeData,
) -> DialogSeverity {
    if variant_contains_any(node, &["info"]) {
        DialogSeverity::Info
    } else if variant_contains_any(node, &["error", "danger"]) {
        DialogSeverity::Error
    } else {
        DialogSeverity::Warning
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn severity_mark_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    match severity(node) {
        DialogSeverity::Info => DIALOG_INFO,
        DialogSeverity::Warning => DIALOG_WARNING,
        DialogSeverity::Error => DIALOG_ERROR,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn severity_border_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    match severity(node) {
        DialogSeverity::Info => DIALOG_INFO_BORDER,
        DialogSeverity::Warning => DIALOG_WARNING_BORDER,
        DialogSeverity::Error => DIALOG_ERROR_BORDER,
    }
}
