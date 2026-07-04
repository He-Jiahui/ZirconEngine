use super::palette::dialog_palette;
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
    let palette = dialog_palette();
    match severity(node) {
        DialogSeverity::Info => palette.info,
        DialogSeverity::Warning => palette.warning,
        DialogSeverity::Error => palette.error,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn severity_border_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = dialog_palette();
    match severity(node) {
        DialogSeverity::Info => palette.info_border,
        DialogSeverity::Warning => palette.warning_border,
        DialogSeverity::Error => palette.error_border,
    }
}
