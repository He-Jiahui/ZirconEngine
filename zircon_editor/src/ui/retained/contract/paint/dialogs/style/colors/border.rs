//! 对话框边框按不可用、严重性装饰和中性容器的合同选择，不随focus/pressed变成按钮焦点环。

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
    } else if kind.uses_severity_chrome() {
        severity_border_color(node)
    } else {
        palette.border
    }
}
