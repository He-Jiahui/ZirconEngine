use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};
use super::super::bounded_extent;

const SKELETON_TEXT_SCALE_Y: f32 = 0.60;
const SKELETON_WAVE_X_RATIO: f32 = 0.28;
const SKELETON_WAVE_WIDTH_RATIO: f32 = 0.22;

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
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn skeleton_default_radius_tracks_host_control_density() {
        let node = TemplatePaneNodeData::default();
        let rect = FrameRect {
            x: 4.0,
            y: 8.0,
            width: 40.0,
            height: 16.0,
        };
        let mut compact = METRICS;
        compact.radius_control = 3.0;

        assert_eq!(skeleton_corner_radius_from_host(&node, &rect, compact), 3.0);
    }

    #[test]
    fn skeleton_frames_stay_inside_tight_parent_bounds() {
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 0.4,
            height: 0.6,
        };
        let mut circular = TemplatePaneNodeData::default();
        circular.component_variant = "circular".to_owned();
        let mut text = TemplatePaneNodeData::default();
        text.component_variant = "text".to_owned();

        for frame in [
            skeleton_frame_for_variant(&circular, &rect),
            skeleton_frame_for_variant(&text, &rect),
            skeleton_wave_frame(&rect),
        ] {
            assert!(frame.x >= rect.x);
            assert!(frame.y >= rect.y);
            assert!(frame.right() <= rect.right());
            assert!(frame.bottom() <= rect.bottom());
        }
    }

    #[test]
    fn skeleton_radius_does_not_exceed_narrow_frame_bounds() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 2.0,
            height: 20.0,
        };

        assert_eq!(
            skeleton_corner_radius_from_host(&TemplatePaneNodeData::default(), &rect, METRICS),
            1.0
        );
    }
}

#[cfg(test)]
#[path = "geometry/single_scan_variant_tests.rs"]
mod single_scan_variant_tests;
