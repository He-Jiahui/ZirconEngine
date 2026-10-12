use zircon_runtime_interface::ui::{
    icon::{UiBuiltinIcon, UiBuiltinIconReference, UiIconSize},
    layout::{UiFrame, UiPoint},
};

use super::geometry::{push_border_with_radius, push_rect_with_radius};
use super::ScreenSpaceUiVertex;

pub(super) fn push_builtin_icon(
    vertices: &mut Vec<ScreenSpaceUiVertex>,
    icon: &str,
    frame: UiFrame,
    color: [f32; 4],
    viewport: UiFrame,
) -> bool {
    let Some(reference) = UiBuiltinIconReference::parse(icon) else {
        return false;
    };

    let icon = reference.icon();
    let icon_frame = icon_frame_for_size(
        frame,
        reference.resolved_size(frame.width.min(frame.height)),
    );
    let extent = icon_frame.width;
    let center = icon_frame.center();
    let stroke = (extent * 0.105).clamp(1.0, 2.5);
    let radius = extent * 0.5;

    match icon {
        UiBuiltinIcon::Info => {
            push_border_with_radius(vertices, icon_frame, stroke, color, radius, viewport);
            let dot = extent * 0.12;
            push_rect_with_radius(
                vertices,
                UiFrame::new(center.x - dot * 0.5, center.y - extent * 0.28, dot, dot),
                color,
                dot * 0.5,
                viewport,
            );
            push_rect_with_radius(
                vertices,
                UiFrame::new(
                    center.x - dot * 0.5,
                    center.y - extent * 0.08,
                    dot,
                    extent * 0.32,
                ),
                color,
                dot * 0.5,
                viewport,
            );
        }
        UiBuiltinIcon::CheckCircle => {
            push_border_with_radius(vertices, icon_frame, stroke, color, radius, viewport);
            push_stroke(
                vertices,
                UiPoint::new(center.x - extent * 0.25, center.y),
                UiPoint::new(center.x - extent * 0.05, center.y + extent * 0.19),
                stroke,
                color,
                viewport,
            );
            push_stroke(
                vertices,
                UiPoint::new(center.x - extent * 0.05, center.y + extent * 0.19),
                UiPoint::new(center.x + extent * 0.29, center.y - extent * 0.22),
                stroke,
                color,
                viewport,
            );
        }
        UiBuiltinIcon::XCircle => {
            push_border_with_radius(vertices, icon_frame, stroke, color, radius, viewport);
            push_stroke(
                vertices,
                UiPoint::new(center.x - extent * 0.2, center.y - extent * 0.2),
                UiPoint::new(center.x + extent * 0.2, center.y + extent * 0.2),
                stroke,
                color,
                viewport,
            );
            push_stroke(
                vertices,
                UiPoint::new(center.x + extent * 0.2, center.y - extent * 0.2),
                UiPoint::new(center.x - extent * 0.2, center.y + extent * 0.2),
                stroke,
                color,
                viewport,
            );
        }
        UiBuiltinIcon::AlertTriangle => {
            let top = UiPoint::new(center.x, icon_frame.y + extent * 0.08);
            let left = UiPoint::new(
                icon_frame.x + extent * 0.12,
                icon_frame.bottom() - extent * 0.13,
            );
            let right = UiPoint::new(
                icon_frame.right() - extent * 0.12,
                icon_frame.bottom() - extent * 0.13,
            );
            push_stroke(vertices, top, left, stroke, color, viewport);
            push_stroke(vertices, left, right, stroke, color, viewport);
            push_stroke(vertices, right, top, stroke, color, viewport);
            let dot = extent * 0.11;
            push_rect_with_radius(
                vertices,
                UiFrame::new(center.x - dot * 0.5, center.y + extent * 0.08, dot, dot),
                color,
                dot * 0.5,
                viewport,
            );
            push_rect_with_radius(
                vertices,
                UiFrame::new(
                    center.x - dot * 0.5,
                    center.y - extent * 0.2,
                    dot,
                    extent * 0.22,
                ),
                color,
                dot * 0.5,
                viewport,
            );
        }
        UiBuiltinIcon::ArrowUp => {
            push_stroke(
                vertices,
                UiPoint::new(center.x, center.y + extent * 0.28),
                UiPoint::new(center.x, center.y - extent * 0.24),
                stroke,
                color,
                viewport,
            );
            push_stroke(
                vertices,
                UiPoint::new(center.x, center.y - extent * 0.24),
                UiPoint::new(center.x - extent * 0.2, center.y - extent * 0.04),
                stroke,
                color,
                viewport,
            );
            push_stroke(
                vertices,
                UiPoint::new(center.x, center.y - extent * 0.24),
                UiPoint::new(center.x + extent * 0.2, center.y - extent * 0.04),
                stroke,
                color,
                viewport,
            );
        }
        UiBuiltinIcon::Plus => {
            push_stroke(
                vertices,
                UiPoint::new(center.x - extent * 0.28, center.y),
                UiPoint::new(center.x + extent * 0.28, center.y),
                stroke,
                color,
                viewport,
            );
            push_stroke(
                vertices,
                UiPoint::new(center.x, center.y - extent * 0.28),
                UiPoint::new(center.x, center.y + extent * 0.28),
                stroke,
                color,
                viewport,
            );
        }
        UiBuiltinIcon::Minus => push_stroke(
            vertices,
            UiPoint::new(center.x - extent * 0.28, center.y),
            UiPoint::new(center.x + extent * 0.28, center.y),
            stroke,
            color,
            viewport,
        ),
        UiBuiltinIcon::Check => {
            push_stroke(
                vertices,
                UiPoint::new(center.x - extent * 0.28, center.y),
                UiPoint::new(center.x - extent * 0.05, center.y + extent * 0.2),
                stroke,
                color,
                viewport,
            );
            push_stroke(
                vertices,
                UiPoint::new(center.x - extent * 0.05, center.y + extent * 0.2),
                UiPoint::new(center.x + extent * 0.3, center.y - extent * 0.23),
                stroke,
                color,
                viewport,
            );
        }
        UiBuiltinIcon::X | UiBuiltinIcon::Close => {
            push_stroke(
                vertices,
                UiPoint::new(center.x - extent * 0.24, center.y - extent * 0.24),
                UiPoint::new(center.x + extent * 0.24, center.y + extent * 0.24),
                stroke,
                color,
                viewport,
            );
            push_stroke(
                vertices,
                UiPoint::new(center.x + extent * 0.24, center.y - extent * 0.24),
                UiPoint::new(center.x - extent * 0.24, center.y + extent * 0.24),
                stroke,
                color,
                viewport,
            );
        }
        UiBuiltinIcon::ChevronDown
        | UiBuiltinIcon::ChevronUp
        | UiBuiltinIcon::ChevronLeft
        | UiBuiltinIcon::ChevronRight => {
            let (first, second) = match icon {
                UiBuiltinIcon::ChevronDown => (
                    UiPoint::new(center.x - extent * 0.22, center.y - extent * 0.1),
                    UiPoint::new(center.x, center.y + extent * 0.14),
                ),
                UiBuiltinIcon::ChevronUp => (
                    UiPoint::new(center.x - extent * 0.22, center.y + extent * 0.1),
                    UiPoint::new(center.x, center.y - extent * 0.14),
                ),
                UiBuiltinIcon::ChevronLeft => (
                    UiPoint::new(center.x + extent * 0.1, center.y - extent * 0.22),
                    UiPoint::new(center.x - extent * 0.14, center.y),
                ),
                _ => (
                    UiPoint::new(center.x - extent * 0.1, center.y - extent * 0.22),
                    UiPoint::new(center.x + extent * 0.14, center.y),
                ),
            };
            let third = match icon {
                UiBuiltinIcon::ChevronDown => {
                    UiPoint::new(center.x + extent * 0.22, center.y - extent * 0.1)
                }
                UiBuiltinIcon::ChevronUp => {
                    UiPoint::new(center.x + extent * 0.22, center.y + extent * 0.1)
                }
                UiBuiltinIcon::ChevronLeft => {
                    UiPoint::new(center.x + extent * 0.1, center.y + extent * 0.22)
                }
                _ => UiPoint::new(center.x - extent * 0.1, center.y + extent * 0.22),
            };
            push_stroke(vertices, first, second, stroke, color, viewport);
            push_stroke(vertices, second, third, stroke, color, viewport);
        }
        UiBuiltinIcon::Search => {
            let circle = UiFrame::new(
                center.x - extent * 0.29,
                center.y - extent * 0.29,
                extent * 0.56,
                extent * 0.56,
            );
            push_border_with_radius(
                vertices,
                circle,
                stroke,
                color,
                circle.width * 0.5,
                viewport,
            );
            push_stroke(
                vertices,
                UiPoint::new(center.x + extent * 0.13, center.y + extent * 0.13),
                UiPoint::new(center.x + extent * 0.31, center.y + extent * 0.31),
                stroke,
                color,
                viewport,
            );
        }
        UiBuiltinIcon::MoreHorizontal | UiBuiltinIcon::MoreVertical => {
            let dot = extent * 0.13;
            for index in 0..3 {
                let offset = (index as f32 - 1.0) * extent * 0.24;
                let (x, y) = if icon == UiBuiltinIcon::MoreHorizontal {
                    (center.x + offset, center.y)
                } else {
                    (center.x, center.y + offset)
                };
                push_rect_with_radius(
                    vertices,
                    UiFrame::new(x - dot * 0.5, y - dot * 0.5, dot, dot),
                    color,
                    dot * 0.5,
                    viewport,
                );
            }
        }
        UiBuiltinIcon::Package => {
            let box_frame = UiFrame::new(
                icon_frame.x + extent * 0.11,
                icon_frame.y + extent * 0.17,
                extent * 0.78,
                extent * 0.69,
            );
            push_border_with_radius(vertices, box_frame, stroke, color, stroke, viewport);
            push_stroke(
                vertices,
                UiPoint::new(box_frame.x, box_frame.y + extent * 0.2),
                UiPoint::new(box_frame.right(), box_frame.y + extent * 0.2),
                stroke,
                color,
                viewport,
            );
            push_stroke(
                vertices,
                UiPoint::new(center.x, box_frame.y + extent * 0.2),
                UiPoint::new(center.x, box_frame.bottom()),
                stroke,
                color,
                viewport,
            );
        }
    }
    true
}

/// Converts an authored icon tier into a centered, local frame inside an
/// already-laid-out icon slot. Callers never provide a screen position here.
pub(super) fn icon_frame_for_slot(icon: Option<&str>, frame: UiFrame) -> UiFrame {
    let available = frame.width.min(frame.height).max(0.0);
    let size = icon
        .and_then(UiBuiltinIconReference::parse)
        .map(|reference| reference.resolved_size(available))
        .unwrap_or_else(|| UiIconSize::nearest_for(available));
    icon_frame_for_size(frame, size)
}

fn icon_frame_for_size(frame: UiFrame, size: UiIconSize) -> UiFrame {
    const ICON_CONTENT_RATIO: f32 = 0.72;

    let available = frame.width.min(frame.height).max(0.0);
    let extent = size.logical_extent().min(available) * ICON_CONTENT_RATIO;
    UiFrame::new(
        frame.x + (frame.width - extent) * 0.5,
        frame.y + (frame.height - extent) * 0.5,
        extent,
        extent,
    )
}

fn push_stroke(
    vertices: &mut Vec<ScreenSpaceUiVertex>,
    start: UiPoint,
    end: UiPoint,
    width: f32,
    color: [f32; 4],
    viewport: UiFrame,
) {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length = (dx * dx + dy * dy).sqrt();
    if !length.is_finite() || length <= f32::EPSILON {
        return;
    }
    let half_width = width.max(0.5) * 0.5;
    let normal = UiPoint::new(-dy / length * half_width, dx / length * half_width);
    push_quad(
        vertices,
        [
            UiPoint::new(start.x + normal.x, start.y + normal.y),
            UiPoint::new(end.x + normal.x, end.y + normal.y),
            UiPoint::new(end.x - normal.x, end.y - normal.y),
            UiPoint::new(start.x - normal.x, start.y - normal.y),
        ],
        color,
        viewport,
    );
}

fn push_quad(
    vertices: &mut Vec<ScreenSpaceUiVertex>,
    points: [UiPoint; 4],
    color: [f32; 4],
    viewport: UiFrame,
) {
    let min_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::INFINITY, f32::min);
    let max_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::INFINITY, f32::min);
    let max_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::NEG_INFINITY, f32::max);
    if !(max_x > min_x && max_y > min_y) {
        return;
    }
    let frame = UiFrame::new(min_x, min_y, max_x - min_x, max_y - min_y);
    let center = frame.center();
    let half_extent = [frame.width * 0.5, frame.height * 0.5];
    let vertices_for_point = |point: UiPoint| ScreenSpaceUiVertex {
        position: [
            (point.x / viewport.width.max(1.0)) * 2.0 - 1.0,
            1.0 - (point.y / viewport.height.max(1.0)) * 2.0,
        ],
        color,
        local_position: [point.x - center.x, point.y - center.y],
        half_extent,
        corner_radius: 0.0,
        border_width: 0.0,
        fill_color: [0.0; 4],
    };
    let [a, b, c, d] = points.map(vertices_for_point);
    vertices.extend_from_slice(&[a, b, c, a, c, d]);
}

#[cfg(test)]
#[path = "tests/icons_unit.rs"]
mod tests;
