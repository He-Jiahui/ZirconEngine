use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};
use super::super::bounded_extent;

const SKELETON_TEXT_SCALE_Y: f32 = 0.60;
const SKELETON_WAVE_X_RATIO: f32 = 0.28;
const SKELETON_WAVE_WIDTH_RATIO: f32 = 0.22;

/// 根节点按 circular/text 词元投影为占位形状；后续圆角与波纹都使用这同一边界。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_frame_for_variant(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    match skeleton_frame_variant(&node.component_variant) {
        1 => {
            let size = bounded_extent(rect.width).min(bounded_extent(rect.height));
            FrameRect {
                x: rect.x + (rect.width - size) * 0.5,
                y: rect.y + (rect.height - size) * 0.5,
                width: size,
                height: size,
            }
        }
        2 => {
            let height = bounded_extent(rect.height) * SKELETON_TEXT_SCALE_Y;
            FrameRect {
                x: rect.x,
                y: rect.y + (rect.height - height) * 0.5,
                width: rect.width,
                height,
            }
        }
        _ => rect.clone(),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_corner_radius(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    skeleton_corner_radius_from_host(node, rect, current_host_metrics())
}

fn skeleton_corner_radius_from_host(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    metrics: HostControlMetrics,
) -> f32 {
    match skeleton_radius_variant(&node.component_variant) {
        1 => return 0.0,
        2 => return bounded_extent(rect.width).min(bounded_extent(rect.height)) * 0.5,
        _ => {}
    }
    let configured =
        configured_corner_radius(node).unwrap_or_else(|| bounded_extent(metrics.radius_control));
    configured
        .min(bounded_extent(rect.width).min(bounded_extent(rect.height)) * 0.5)
        .max(0.0)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_wave_frame(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + bounded_extent(rect.width) * SKELETON_WAVE_X_RATIO,
        y: rect.y,
        width: bounded_extent(rect.width) * SKELETON_WAVE_WIDTH_RATIO,
        height: bounded_extent(rect.height),
    }
}

fn configured_corner_radius(node: &TemplatePaneNodeData) -> Option<f32> {
    let radius = node
        .button_style
        .element
        .corner_radius
        .max(node.corner_radius);
    (radius.is_finite() && radius > 0.0).then_some(radius)
}

fn skeleton_frame_variant(component_variant: &str) -> u8 {
    let mut circular = false;
    let mut text = false;
    for part in component_variant.split(|character: char| {
        character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
    }) {
        circular |= part.eq_ignore_ascii_case("circular");
        text |= part.eq_ignore_ascii_case("text");
    }
    if circular {
        1
    } else if text {
        2
    } else {
        0
    }
}

fn skeleton_radius_variant(component_variant: &str) -> u8 {
    let mut rectangular = false;
    let mut circular = false;
    for part in component_variant.split(|character: char| {
        character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
    }) {
        rectangular |= part.eq_ignore_ascii_case("rectangular");
        circular |= part.eq_ignore_ascii_case("circular");
    }
    if rectangular {
        1
    } else if circular {
        2
    } else {
        0
    }
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;

#[cfg(test)]
#[path = "geometry/tests/single_scan_variant_tests.rs"]
mod single_scan_variant_tests;
