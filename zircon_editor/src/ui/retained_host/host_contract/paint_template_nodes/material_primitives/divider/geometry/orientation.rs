use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_is_vertical(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> bool {
    let (has_vertical, has_horizontal) = divider_orientation_flags(&node.component_variant);
    has_vertical || (!has_horizontal && rect.height > rect.width * 1.4)
}

fn divider_orientation_flags(component_variant: &str) -> (bool, bool) {
    let mut has_vertical = false;
    let mut has_horizontal = false;
    for part in component_variant.split(|character: char| {
        character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
    }) {
        has_vertical |=
            part.eq_ignore_ascii_case("vertical") || part.eq_ignore_ascii_case("wrapperVertical");
        has_horizontal |= part.eq_ignore_ascii_case("horizontal");
    }
    (has_vertical, has_horizontal)
}

#[cfg(test)]
#[path = "orientation/single_scan_variant_tests.rs"]
mod single_scan_variant_tests;
