use super::super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::component_variant_contains;

pub(super) fn divider_text_align(node: &TemplatePaneNodeData) -> DividerTextAlign {
    if component_variant_contains(node, "textAlignRight")
        || component_variant_contains(node, "right")
        || matches!(node.text_align.as_str(), "right" | "end")
    {
        DividerTextAlign::Right
    } else if component_variant_contains(node, "textAlignLeft")
        || component_variant_contains(node, "left")
        || matches!(node.text_align.as_str(), "left" | "start")
    {
        DividerTextAlign::Left
    } else {
        DividerTextAlign::Center
    }
}

pub(super) enum DividerTextAlign {
    Left,
    Center,
    Right,
}
