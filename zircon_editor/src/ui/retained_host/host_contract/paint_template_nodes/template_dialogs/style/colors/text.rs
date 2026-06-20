use super::super::super::identity::DialogKind;
use super::super::severity::{severity, severity_mark_color, DialogSeverity};
use super::super::tokens::{DIALOG_BODY, DIALOG_DISABLED_TEXT, DIALOG_TITLE};
use super::super::variants::variant_contains_any;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_title_color(
    node: &TemplatePaneNodeData,
    kind: DialogKind,
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_TEXT
    } else if matches!(kind, DialogKind::ConfirmDialog)
        && (variant_contains_any(node, &["destructive"])
            || matches!(severity(node), DialogSeverity::Error))
    {
        severity_mark_color(node)
    } else {
        DIALOG_TITLE
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_body_color(
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_TEXT
    } else {
        DIALOG_BODY
    }
}
