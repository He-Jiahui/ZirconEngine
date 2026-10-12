// 专用认领依据 canvas 语义与 timeline-strip 变体，不依赖具体工作台资源 ID。
use super::super::super::data::TemplatePaneNodeData;

pub(super) fn is_timeline_strip(node: &TemplatePaneNodeData) -> bool {
    node.component_role.as_str() == "canvas"
        && node
            .component_variant
            .split_whitespace()
            .any(|token| token == "timeline-strip")
}
