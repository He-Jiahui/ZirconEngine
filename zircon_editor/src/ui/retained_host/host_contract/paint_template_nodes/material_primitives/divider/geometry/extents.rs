use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::component_variant_contains;
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
    if component_variant_contains(node, "middle") {
        let inset = divider_middle_horizontal_inset().min(rect.width * 0.45);
        start += inset;
        end -= inset;
    } else if component_variant_contains(node, "inset") {
        let inset = divider_inset_horizontal_inset().min(rect.width * 0.9);
        start += inset;
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
    if component_variant_contains(node, "middle") {
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
