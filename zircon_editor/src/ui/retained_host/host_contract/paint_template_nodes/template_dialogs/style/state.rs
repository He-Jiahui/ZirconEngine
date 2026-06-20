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
