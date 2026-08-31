use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::align::pixel_aligned;
use super::metrics::{
    divider_inset_horizontal_inset, divider_middle_horizontal_inset, divider_middle_vertical_inset,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn horizontal_divider_extent(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> (f32, f32) {
    let mut start = rect.x;
    let mut end = rect.x + rect.width;
    match divider_extent_variant(&node.component_variant) {
        DividerExtentVariant::Middle => {
            let inset = divider_middle_horizontal_inset().min(rect.width * 0.45);
            start += inset;
            end -= inset;
        }
        DividerExtentVariant::Inset => {
            let inset = divider_inset_horizontal_inset().min(rect.width * 0.9);
            start += inset;
        }
        DividerExtentVariant::None => {}
    }
    if end < start {
        let center = rect.x + rect.width * 0.5;
        (center, center)
    } else {
        (pixel_aligned(start), pixel_aligned(end))
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn vertical_divider_extent(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> (f32, f32) {
    let mut top = rect.y;
    let mut bottom = rect.y + rect.height;
    if divider_extent_variant(&node.component_variant) == DividerExtentVariant::Middle {
        let inset = divider_middle_vertical_inset().min(rect.height * 0.45);
        top += inset;
        bottom -= inset;
    }
    if bottom < top {
        let center = rect.y + rect.height * 0.5;
        (center, center)
    } else {
        (pixel_aligned(top), pixel_aligned(bottom))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DividerExtentVariant {
    #[default]
    None,
    Inset,
    Middle,
}

fn divider_extent_variant(component_variant: &str) -> DividerExtentVariant {
    let mut variant = DividerExtentVariant::None;
    for part in component_variant.split(|character: char| {
        character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
    }) {
        if part.eq_ignore_ascii_case("middle") {
            return DividerExtentVariant::Middle;
        }
        if part.eq_ignore_ascii_case("inset") {
            variant = DividerExtentVariant::Inset;
        }
    }
    variant
}

#[cfg(test)]
#[path = "extents/single_scan_variant_tests.rs"]
mod single_scan_variant_tests;
