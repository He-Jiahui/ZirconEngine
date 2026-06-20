use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::component_variant_contains;
use super::metrics::BADGE_CIRCULAR_OFFSET_RATIO;

pub(super) fn badge_anchor_point(node: &TemplatePaneNodeData, rect: &FrameRect) -> (f32, f32) {
    let circular = component_variant_contains(node, "circular")
        || component_variant_contains(node, "overlapCircular");
    let offset_x = if circular {
        rect.width * BADGE_CIRCULAR_OFFSET_RATIO
    } else {
        0.0
    };
    let offset_y = if circular {
        rect.height * BADGE_CIRCULAR_OFFSET_RATIO
    } else {
        0.0
    };
    let left = badge_is_left_anchored(node);
    let bottom = badge_is_bottom_anchored(node);
    let x = if left {
        rect.x + offset_x
    } else {
        rect.x + rect.width - offset_x
    };
    let y = if bottom {
        rect.y + rect.height - offset_y
    } else {
        rect.y + offset_y
    };
    (x, y)
}

fn badge_is_left_anchored(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "left")
        || component_variant_contains(node, "anchorOriginTopLeft")
        || component_variant_contains(node, "anchorOriginBottomLeft")
}

fn badge_is_bottom_anchored(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "bottom")
        || component_variant_contains(node, "anchorOriginBottomLeft")
        || component_variant_contains(node, "anchorOriginBottomRight")
}
