//! 进度值已由节点转换层归一化；这里只呈现轨道与填充，不回写数值或推进动画。
//! 不确定状态使用固定双段示意，命令层级让填充覆盖轨道并保留上游裁剪。

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_style::template_corner_radius;
use super::super::metrics::{linear_progress_radius, material_feedback_metrics};
use super::super::state::{
    progress_fill_color, progress_is_indeterminate, progress_percent, progress_track_color,
};
use crate::ui::retained_host::host_contract::paint_geometry::{
    bounded_extent, corner_radius_for_frame, intersect,
};

const PROGRESS_SEGMENT_RATIO_UNITS: f32 = 100.0;
const LINEAR_PROGRESS_COMMAND_CAPACITY: usize = 3;

// 不确定进度的两个固定段共用轨道比例坐标，允许在不同密度与宽度下复用。
#[derive(Clone, Copy)]
struct IndeterminateProgressSegmentSpec {
    x_units: u8,
    width_units: u8,
}

impl IndeterminateProgressSegmentSpec {
    const fn new(x_units: u8, width_units: u8) -> Self {
        Self {
            x_units,
            width_units,
        }
    }
}

const INDETERMINATE_SEGMENTS: [IndeterminateProgressSegmentSpec; 2] = [
    IndeterminateProgressSegmentSpec::new(12, 36),
    IndeterminateProgressSegmentSpec::new(62, 24),
];

pub(super) fn push_linear_progress_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if bounded_extent(rect.width) <= 0.0
        || bounded_extent(rect.height) <= 0.0
        || intersect(rect, clip).is_none()
    {
        return;
    }
    commands.reserve(LINEAR_PROGRESS_COMMAND_CAPACITY);
    let track_radius = corner_radius_for_frame(
        rect,
        linear_progress_radius(
            template_corner_radius(node),
            rect.height,
            material_feedback_metrics(),
        ),
    );
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(progress_track_color(node)),
        None,
        0.0,
        track_radius,
        opacity,
    ));

    let fill = progress_fill_color(node);
    if progress_is_indeterminate(node) {
        for segment in INDETERMINATE_SEGMENTS {
            let bar = indeterminate_segment_rect(rect, segment);
            if bar.width <= 0.0 || bar.height <= 0.0 {
                continue;
            }
            if intersect(&bar, clip).is_none() {
                continue;
            }
            commands.push(HostPaintCommand::quad(
                bar.clone(),
                Some(clip.clone()),
                order + 1,
                Some(fill),
                None,
                0.0,
                corner_radius_for_frame(&bar, track_radius),
                opacity,
            ));
        }
        return;
    }

    let Some(fill_rect) = determinate_fill_rect(rect, progress_percent(node)) else {
        return;
    };
    if intersect(&fill_rect, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::quad(
        fill_rect.clone(),
        Some(clip.clone()),
        order + 1,
        Some(fill),
        None,
        0.0,
        corner_radius_for_frame(&fill_rect, track_radius),
        opacity,
    ));
}

fn determinate_fill_rect(rect: &FrameRect, percent: f32) -> Option<FrameRect> {
    let track_width = bounded_extent(rect.width);
    let fraction = if percent.is_finite() {
        percent.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let width = track_width * fraction;
    (width > 0.0).then_some(FrameRect {
        x: rect.x,
        y: rect.y,
        width,
        height: bounded_extent(rect.height),
    })
}

fn indeterminate_segment_rect(
    rect: &FrameRect,
    segment: IndeterminateProgressSegmentSpec,
) -> FrameRect {
    let track_width = bounded_extent(rect.width);
    let offset = track_width * f32::from(segment.x_units) / PROGRESS_SEGMENT_RATIO_UNITS;
    let requested_width =
        track_width * f32::from(segment.width_units) / PROGRESS_SEGMENT_RATIO_UNITS;
    FrameRect {
        x: rect.x + offset,
        y: rect.y,
        width: requested_width.min((track_width - offset).max(0.0)),
        height: bounded_extent(rect.height),
    }
}

#[cfg(test)]
#[path = "tests/linear.rs"]
mod tests;
