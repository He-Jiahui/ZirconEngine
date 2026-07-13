use super::super::super::data::TemplatePaneNodeData;

pub(super) fn is_timeline_strip(node: &TemplatePaneNodeData) -> bool {
    node.component_role.as_str() == "canvas"
        && node
            .component_variant
            .split_whitespace()
            .any(|token| token == "timeline-strip")
}
