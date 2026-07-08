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

const DIVIDER_TEXT_ALIGN_LEFT_RATIO: f32 = 0.1;
const DIVIDER_TEXT_ALIGN_CENTER_RATIO: f32 = 0.5;
const DIVIDER_TEXT_ALIGN_RIGHT_RATIO: f32 = 0.9;

pub(super) fn divider_text_align_ratio(align: DividerTextAlign) -> f32 {
    match align {
        DividerTextAlign::Left => DIVIDER_TEXT_ALIGN_LEFT_RATIO,
        DividerTextAlign::Center => DIVIDER_TEXT_ALIGN_CENTER_RATIO,
        DividerTextAlign::Right => DIVIDER_TEXT_ALIGN_RIGHT_RATIO,
    }
}
