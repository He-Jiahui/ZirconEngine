use super::super::super::identity::DialogKind;
use super::super::severity::severity_border_color;
use super::super::tokens::{DIALOG_ACTIVE_BORDER, DIALOG_BORDER, DIALOG_DISABLED_BORDER};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_border_color(
    node: &TemplatePaneNodeData,
    kind: DialogKind,
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_BORDER
    } else if matches!(kind, DialogKind::ConfirmDialog) {
        severity_border_color(node)
    } else if node.focused || node.pressed || node.popup_open {
        DIALOG_ACTIVE_BORDER
    } else {
        DIALOG_BORDER
    }
}
