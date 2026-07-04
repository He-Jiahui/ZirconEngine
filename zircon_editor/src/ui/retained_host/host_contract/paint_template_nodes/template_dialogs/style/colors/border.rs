use super::super::super::identity::DialogKind;
use super::super::palette::dialog_palette;
use super::super::severity::severity_border_color;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_border_color(
    node: &TemplatePaneNodeData,
    kind: DialogKind,
    unavailable: bool,
) -> [u8; 4] {
    let palette = dialog_palette();
    if unavailable {
        palette.disabled_border
    } else if matches!(kind, DialogKind::ConfirmDialog) {
        severity_border_color(node)
    } else if node.focused || node.pressed || node.popup_open {
        palette.active_border
    } else {
        palette.border
    }
}
