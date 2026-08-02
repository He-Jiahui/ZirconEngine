use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

use super::super::super::identity::{chip_has_avatar, chip_has_icon, chip_is_small};
use super::super::metrics::{
    CHIP_AVATAR_MEDIUM_EDGE, CHIP_AVATAR_SMALL_EDGE, CHIP_ICON_MEDIUM_EDGE, CHIP_ICON_SMALL_EDGE,
    chip_bounded_extent,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_leading_edge(
    node: &TemplatePaneNodeData,
) -> f32 {
    if chip_has_avatar(node) {
        if chip_is_small(node) {
            CHIP_AVATAR_SMALL_EDGE
        } else {
            CHIP_AVATAR_MEDIUM_EDGE
        }
    } else if chip_has_icon(node) {
        if chip_is_small(node) {
            CHIP_ICON_SMALL_EDGE
        } else {
            CHIP_ICON_MEDIUM_EDGE
        }
    } else {
        0.0
    }
}

pub(super) fn chip_avatar_edge(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let desired = if chip_is_small(node) {
        CHIP_AVATAR_SMALL_EDGE
    } else {
        CHIP_AVATAR_MEDIUM_EDGE
    };
    chip_child_edge(desired, rect)
}

pub(super) fn chip_icon_edge(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let desired = if chip_is_small(node) {
        CHIP_ICON_SMALL_EDGE
    } else {
        CHIP_ICON_MEDIUM_EDGE
    };
    chip_child_edge(desired, rect)
}

fn chip_child_edge(desired: f32, rect: &FrameRect) -> f32 {
    let width = chip_bounded_extent(rect.width);
    let content_height = (chip_bounded_extent(rect.height) - 4.0).max(0.0);
    desired.min(width).min(content_height)
}
