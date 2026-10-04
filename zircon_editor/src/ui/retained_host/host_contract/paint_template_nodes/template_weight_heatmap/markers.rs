// 动态 source marker 在静态强度场之后叠加；已选择状态影响标记大小/颜色而无需重新计算静态场。
use crate::ui::weight_heatmap::WeightHeatmapSource;

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::geometry::WeightHeatmapGeometry;
use super::palette::{SELECTED_SOURCE, SOURCE_MARKER};

/// source 列表来自已归一化 generation；只在 plot 可画时预留列表容量。单个标记的透明裁剪仍交给 HostPaintCommand。
pub(super) fn push_heat_source_markers(
    commands: &mut Vec<HostPaintCommand>,
    sources: &[WeightHeatmapSource],
    geometry: &WeightHeatmapGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !geometry.is_drawable() {
        return;
    }
    commands.reserve(sources.len());
    for source in sources {
        let x = geometry.x_for_normalized(source.x());
        let y = geometry.y_for_normalized(source.y());
        push_source_marker(
            commands,
            x,
            y,
            if source.selected() { 5.0 } else { 3.0 },
            if source.selected() {
                SELECTED_SOURCE
            } else {
                SOURCE_MARKER
            },
            clip,
            order + 4,
            opacity,
        );
    }
}

fn push_source_marker(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    y: f32,
    radius: f32,
    color: [u8; 4],
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: x - radius,
            y: y - radius,
            width: radius * 2.0 + 1.0,
            height: radius * 2.0 + 1.0,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        radius,
        opacity,
    ));
}

#[cfg(test)]
#[path = "tests/markers.rs"]
mod tests;

#[cfg(test)]
#[path = "markers/tests/capacity_tests.rs"]
mod capacity_tests;
