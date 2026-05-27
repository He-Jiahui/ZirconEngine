use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::theme::PALETTE;
use super::{component_variant_contains, resolved_style_color};

const MUI_PAPER_DARK_BACKGROUND: [u8; 4] = [18, 18, 18, 255];
const MUI_PAPER_DARK_DIVIDER: [u8; 4] = [255, 255, 255, 31];
const MUI_SHADOW_UMBRA: [u8; 4] = [0, 0, 0, 51];
const MUI_SHADOW_PENUMBRA: [u8; 4] = [0, 0, 0, 36];
const MUI_SHADOW_AMBIENT: [u8; 4] = [0, 0, 0, 31];
const MUI_PAPER_DEFAULT_RADIUS: f32 = 4.0;

struct ShadowLayer {
    offset_y: f32,
    grow: f32,
    color: [u8; 4],
}

pub(super) fn push_paper_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_paper_root_node(node) {
        return false;
    }

    let paper_rect = pixel_aligned_rect(rect);
    if paper_rect.width <= 0.0 || paper_rect.height <= 0.0 {
        return true;
    }

    let outlined = paper_is_outlined(node);
    let elevation = paper_elevation(node);
    let corner_radius = paper_corner_radius(node, &paper_rect);

    if !outlined && elevation > 0.0 {
        push_paper_shadow(
            commands,
            &paper_rect,
            clip,
            order - 3,
            elevation,
            corner_radius,
            opacity,
        );
    }

    commands.push(HostPaintCommand::quad(
        paper_rect.clone(),
        Some(clip.clone()),
        order,
        Some(paper_background_color(node)),
        paper_border_color(node, outlined),
        paper_border_width(node, outlined),
        corner_radius,
        opacity,
    ));

    if !outlined && elevation > 0.0 && paper_background_color(node)[3] > 0 {
        commands.push(HostPaintCommand::quad(
            paper_rect,
            Some(clip.clone()),
            order + 1,
            Some(paper_dark_overlay(elevation)),
            None,
            0.0,
            corner_radius,
            opacity,
        ));
    }

    true
}

fn is_paper_root_node(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "paper" | "Paper" | "mui-paper" | "MuiPaper"
    ) || matches!(node.role.as_str(), "Paper" | "MuiPaper")
}

fn push_paper_shadow(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    elevation: f32,
    corner_radius: f32,
    opacity: f32,
) {
    for (index, layer) in shadow_layers(elevation).into_iter().enumerate() {
        commands.push(HostPaintCommand::quad(
            expanded_offset_rect(rect, layer.offset_y, layer.grow),
            Some(clip.clone()),
            order + index as i32,
            Some(layer.color),
            None,
            0.0,
            (corner_radius + layer.grow).max(0.0),
            opacity,
        ));
    }
}

fn shadow_layers(elevation: f32) -> [ShadowLayer; 3] {
    let elevation = elevation.clamp(1.0, 24.0);
    let offset = elevation.round().max(1.0);
    [
        ShadowLayer {
            offset_y: (elevation / 3.0).round().max(1.0),
            grow: 1.0,
            color: MUI_SHADOW_AMBIENT,
        },
        ShadowLayer {
            offset_y: offset,
            grow: 0.0,
            color: MUI_SHADOW_PENUMBRA,
        },
        ShadowLayer {
            offset_y: offset,
            grow: 0.0,
            color: MUI_SHADOW_UMBRA,
        },
    ]
}

fn expanded_offset_rect(rect: &FrameRect, offset_y: f32, grow: f32) -> FrameRect {
    FrameRect {
        x: rect.x - grow,
        y: rect.y + offset_y - grow,
        width: rect.width + grow * 2.0,
        height: rect.height + grow * 2.0,
    }
}

fn paper_is_outlined(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "outlined")
        || node.surface_variant.as_str() == "paper-outlined"
}

fn paper_elevation(node: &TemplatePaneNodeData) -> f32 {
    if node.elevation.is_finite() {
        node.elevation.max(0.0)
    } else {
        0.0
    }
}

fn paper_corner_radius(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    if component_variant_contains(node, "square") {
        return 0.0;
    }
    let configured = node
        .corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0);
    let radius = if configured > 0.0 {
        configured
    } else {
        MUI_PAPER_DEFAULT_RADIUS
    };
    radius.min(rect.width.min(rect.height) * 0.5)
}

fn paper_background_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .unwrap_or(MUI_PAPER_DARK_BACKGROUND)
}

fn paper_border_color(node: &TemplatePaneNodeData, outlined: bool) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| (paper_border_width(node, outlined) > 0.0).then_some(MUI_PAPER_DARK_DIVIDER))
}

fn paper_border_width(node: &TemplatePaneNodeData, outlined: bool) -> f32 {
    let configured = node
        .border_width
        .max(node.button_style.element.border_width)
        .max(0.0);
    if outlined {
        configured.max(1.0)
    } else {
        configured
    }
}

fn paper_dark_overlay(elevation: f32) -> [u8; 4] {
    let alpha = get_overlay_alpha(elevation);
    [255, 255, 255, (alpha * 255.0).round() as u8]
}

fn get_overlay_alpha(elevation: f32) -> f32 {
    if elevation < 1.0 {
        5.11916 * elevation.powi(2)
    } else {
        let alpha_value = 4.5 * (elevation + 1.0).ln() + 2.0;
        (alpha_value * 10.0).round() / 1000.0
    }
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}
