use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::{component_variant_contains, resolved_style_color};

const SKELETON_DEFAULT_BG: [u8; 4] = [58, 66, 73, 255];
const SKELETON_WAVE: [u8; 4] = [255, 255, 255, 36];
const SKELETON_DISABLED_OPACITY: f32 = 0.56;
const SKELETON_TEXT_SCALE_Y: f32 = 0.60;
const SKELETON_DEFAULT_RADIUS: f32 = 4.0;
const SKELETON_WAVE_X_RATIO: f32 = 0.28;
const SKELETON_WAVE_WIDTH_RATIO: f32 = 0.22;

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
            FrameRect {
                x: skeleton_rect.x + skeleton_rect.width * SKELETON_WAVE_X_RATIO,
                y: skeleton_rect.y,
                width: (skeleton_rect.width * SKELETON_WAVE_WIDTH_RATIO).max(1.0),
                height: skeleton_rect.height,
            },
            Some(clip.clone()),
            order + 1,
            Some(SKELETON_WAVE),
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

fn skeleton_frame_for_variant(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    if component_variant_contains(node, "circular") {
        let size = rect.width.min(rect.height).max(1.0);
        return FrameRect {
            x: rect.x + (rect.width - size) * 0.5,
            y: rect.y + (rect.height - size) * 0.5,
            width: size,
            height: size,
        };
    }
    if component_variant_contains(node, "text") {
        let height = (rect.height * SKELETON_TEXT_SCALE_Y).max(1.0);
        return FrameRect {
            x: rect.x,
            y: rect.y + (rect.height - height) * 0.5,
            width: rect.width,
            height,
        };
    }
    rect.clone()
}

fn skeleton_corner_radius(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    if component_variant_contains(node, "rectangular") {
        return 0.0;
    }
    if component_variant_contains(node, "circular") {
        return rect.width.min(rect.height) * 0.5;
    }
    let configured = configured_corner_radius(node).unwrap_or(SKELETON_DEFAULT_RADIUS);
    configured.min(rect.height * 0.5).max(0.0)
}

fn configured_corner_radius(node: &TemplatePaneNodeData) -> Option<f32> {
    let radius = node
        .button_style
        .element
        .corner_radius
        .max(node.corner_radius);
    (radius.is_finite() && radius > 0.0).then_some(radius)
}

fn skeleton_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .unwrap_or(SKELETON_DEFAULT_BG)
}

fn skeleton_border_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| (skeleton_border_width(node) > 0.0).then_some(SKELETON_DEFAULT_BG))
}

fn skeleton_border_width(node: &TemplatePaneNodeData) -> f32 {
    let width = node
        .button_style
        .element
        .border_width
        .max(node.border_width);
    if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    }
}

fn skeleton_opacity(node: &TemplatePaneNodeData) -> f32 {
    if node.disabled {
        SKELETON_DISABLED_OPACITY
    } else {
        1.0
    }
}
