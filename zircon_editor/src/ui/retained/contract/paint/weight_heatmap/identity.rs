// 只允许 canvas+weight-heatmap 变体由热图入口认领，与工作台 control_id 无关。
use super::super::super::data::TemplatePaneNodeData;

pub(super) fn is_weight_heatmap(node: &TemplatePaneNodeData) -> bool {
    node.component_role.as_str() == "canvas"
        && node
            .component_variant
            .split_whitespace()
            .any(|token| token == "weight-heatmap")
}
