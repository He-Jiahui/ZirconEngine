//! 把dialog投影token转为呈现可用性；confirmDisabled只影响确认动作，整体disabled/loading token影响整个对话框。
//! 这些结果供painter显示禁用状态，不能替代交互owner对操作可执行性的检查。

use super::variants::variant_contains_any;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_unavailable(
    node: &TemplatePaneNodeData,
) -> bool {
    node.disabled || variant_contains_any(node, &["disabled", "loading"])
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn confirm_enabled(
    node: &TemplatePaneNodeData,
) -> bool {
    !variant_contains_any(
        node,
        &[
            "confirmDisabled",
            "confirm-disabled",
            "confirm_disabled",
            "disabledConfirm",
        ],
    )
}
