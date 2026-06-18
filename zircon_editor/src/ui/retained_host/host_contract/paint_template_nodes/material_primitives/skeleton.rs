use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::component_variant_contains;
use geometry::{skeleton_corner_radius, skeleton_frame_for_variant, skeleton_wave_frame};
use style::{
    skeleton_border_color, skeleton_border_width, skeleton_color, skeleton_opacity,
    skeleton_wave_color,
};

mod geometry;
mod style;

pub(super) fn push_skeleton_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_skeleton_child_node(node) {
        return true;
    }
    if !is_skeleton_root_node(node) {
        return false;
    }

    let skeleton_rect = skeleton_frame_for_variant(node, rect);
    if skeleton_rect.width <= 0.0 || skeleton_rect.height <= 0.0 {
        return true;
    }

    let radius = skeleton_corner_radius(node, &skeleton_rect);
    let effective_opacity = skeleton_opacity(node) * opacity;
    commands.push(HostPaintCommand::quad(
        skeleton_rect.clone(),
        Some(clip.clone()),
        order,
        Some(skeleton_color(node)),
        skeleton_border_color(node),
        skeleton_border_width(node),
        radius,
        effective_opacity,
    ));

    if component_variant_contains(node, "wave") {
        commands.push(HostPaintCommand::quad(
            skeleton_wave_frame(&skeleton_rect),
            Some(clip.clone()),
            order + 1,
            Some(skeleton_wave_color()),
            None,
            0.0,
            radius,
            effective_opacity,
        ));
    }

    true
}

fn is_skeleton_root_node(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "skeleton" | "Skeleton" | "mui-skeleton" | "MuiSkeleton"
    ) || matches!(node.role.as_str(), "Skeleton" | "MuiSkeleton")
}

fn is_skeleton_child_node(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "muiSkeletonChild")
        || component_variant_contains(node, "SkeletonChild")
        || component_variant_contains(node, "skeletonChild")
}
