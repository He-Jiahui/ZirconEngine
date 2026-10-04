// 仅 canvas+sample-grid 语义属于专用绘制链；资源的 control_id 不决定认领，避免演示画布被通用容器覆盖。
use super::super::super::data::TemplatePaneNodeData;

pub(super) fn is_sample_grid(node: &TemplatePaneNodeData) -> bool {
    node.component_role.as_str() == "canvas"
        && node
            .component_variant
            .split_whitespace()
            .any(|token| token == "sample-grid")
}
