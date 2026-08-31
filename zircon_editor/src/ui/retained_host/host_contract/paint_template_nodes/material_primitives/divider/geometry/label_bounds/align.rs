use super::super::super::super::super::super::data::TemplatePaneNodeData;

pub(super) fn divider_text_align(node: &TemplatePaneNodeData) -> DividerTextAlign {
    divider_text_align_for_variant(&node.component_variant, &node.text_align)
}

fn divider_text_align_for_variant(component_variant: &str, text_align: &str) -> DividerTextAlign {
    let mut has_right = false;
    let mut has_left = false;
    for part in component_variant.split(|character: char| {
        character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
    }) {
        has_right |=
            part.eq_ignore_ascii_case("textAlignRight") || part.eq_ignore_ascii_case("right");
        has_left |= part.eq_ignore_ascii_case("textAlignLeft") || part.eq_ignore_ascii_case("left");
    }
    if has_right || matches!(text_align, "right" | "end") {
        DividerTextAlign::Right
    } else if has_left || matches!(text_align, "left" | "start") {
        DividerTextAlign::Left
    } else {
        DividerTextAlign::Center
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[cfg(test)]
#[path = "align/single_scan_variant_tests.rs"]
mod single_scan_variant_tests;
