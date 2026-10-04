// 将生成数据的借用标签落到 runtime 文本命令；测量、相邻刻度空间与帧门槛决定是否生成文字，所有权只在命令实际入队时转移。
use std::borrow::Cow;

use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_text::measure_runtime_text_width;
use super::super::render_commands::HostPaintCommand;
use super::geometry::SampleGridGeometry;
use super::metrics::{
    AXIS_FONT_SIZE, AXIS_LINE_HEIGHT, AXIS_TITLE_EDGE_INSET, AXIS_TITLE_GAP, TICK_FONT_SIZE,
    TICK_LINE_HEIGHT, X_TICK_PLOT_GAP,
};
use super::palette::SampleGridPalette;

#[cfg(test)]
#[path = "text/tests/capacity_tests.rs"]
mod capacity_tests;

pub(super) fn push_sample_grid_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    geometry: &SampleGridGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: SampleGridPalette,
) {
    let grid = &node.sample_grid.generation;
    let x_tick_frames = grid
        .x_ticks()
        .iter()
        .map(|tick| {
            let x = geometry.x_for_value(tick.value(), grid.x_min(), grid.x_max());
            let tick_width = measured_text_frame_width(
                tick.label(),
                TICK_FONT_SIZE,
                geometry.plot.width.min(48.0),
            );
            FrameRect {
                x: (x - tick_width * 0.5).clamp(
                    geometry.plot.x,
                    (geometry.plot.right() - tick_width).max(geometry.plot.x),
                ),
                y: geometry.plot.y - TICK_LINE_HEIGHT - X_TICK_PLOT_GAP,
                width: tick_width,
                height: TICK_LINE_HEIGHT,
            }
        })
        .collect::<Vec<_>>();
    // TODO: [CR-EDITOR-PAINT-SAMPLE-0001] 此相邻检查按输入顺序推断屏幕顺序；投影器只过滤数值，不保证 x_ticks 升序。
    // 目前正式 ZUI 按升序声明。若运行时允许乱序刻度，需在数据入口明确排序/拒绝，或在此按绘制坐标判断重叠。
    if x_tick_frames
        .windows(2)
        .all(|pair| pair[0].right() + AXIS_TITLE_GAP <= pair[1].x)
    {
        for (tick, frame) in grid.x_ticks().iter().zip(x_tick_frames) {
            push_text(
                commands,
                frame,
                clip,
                order + 4,
                tick.label(),
                palette.tick_text,
                TICK_FONT_SIZE,
                TICK_LINE_HEIGHT,
                opacity,
            );
        }
    }
    for tick in grid.y_ticks() {
        let y = geometry.y_for_value(tick.value(), grid.y_min(), grid.y_max());
        let available_width = (geometry.plot.x - geometry.outer.x - X_TICK_PLOT_GAP).max(0.0);
        let tick_width = measured_text_frame_width(tick.label(), TICK_FONT_SIZE, available_width);
        push_text(
            commands,
            FrameRect {
                x: geometry.plot.x - X_TICK_PLOT_GAP - tick_width,
                y: y - TICK_LINE_HEIGHT * 0.5,
                width: tick_width,
                height: TICK_LINE_HEIGHT,
            },
            clip,
            order + 4,
            tick.label(),
            palette.tick_text,
            TICK_FONT_SIZE,
            TICK_LINE_HEIGHT,
            opacity,
        );
    }

    let x_axis_width =
        measured_text_frame_width(grid.x_axis_label(), AXIS_FONT_SIZE, geometry.plot.width);
    let x_axis_x = geometry.plot.x + (geometry.plot.width - x_axis_width).max(0.0) * 0.5;
    if !grid.y_axis_label().trim().is_empty() {
        let y_axis_x = geometry.outer.x + AXIS_TITLE_EDGE_INSET;
        let available_width = (x_axis_x - AXIS_TITLE_GAP - y_axis_x).max(0.0);
        let y_axis_width =
            measured_text_frame_width(grid.y_axis_label(), AXIS_FONT_SIZE, available_width);
        push_text(
            commands,
            FrameRect {
                x: y_axis_x,
                y: geometry.outer.y + AXIS_TITLE_EDGE_INSET,
                width: y_axis_width,
                height: AXIS_LINE_HEIGHT,
            },
            clip,
            order + 5,
            grid.y_axis_label(),
            palette.axis_text,
            AXIS_FONT_SIZE,
            AXIS_LINE_HEIGHT,
            opacity,
        );
    }
    if !grid.x_axis_label().trim().is_empty() {
        push_text(
            commands,
            FrameRect {
                x: x_axis_x,
                y: geometry.outer.y + AXIS_TITLE_EDGE_INSET,
                width: x_axis_width,
                height: AXIS_LINE_HEIGHT,
            },
            clip,
            order + 5,
            grid.x_axis_label(),
            palette.axis_text,
            AXIS_FONT_SIZE,
            AXIS_LINE_HEIGHT,
            opacity,
        );
    }
}

fn measured_text_frame_width(text: &str, font_size: f32, available_width: f32) -> f32 {
    if !available_width.is_finite() || available_width <= f32::EPSILON {
        return 0.0;
    }
    (measure_runtime_text_width(text, font_size) + 2.0).min(available_width)
}

/// 网格刻度、轴标题及选中点标签共用的最后一道文字帧门槛；借用文本到真正发命令后才转为拥有值。
pub(super) fn push_text<'a>(
    commands: &mut Vec<HostPaintCommand>,
    frame: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: impl Into<Cow<'a, str>>,
    color: [u8; 4],
    font_size: f32,
    line_height: f32,
    opacity: f32,
) {
    let text = text.into();
    if text.trim().is_empty()
        || !frame.x.is_finite()
        || !frame.y.is_finite()
        || !frame.width.is_finite()
        || !frame.height.is_finite()
        || frame.width <= f32::EPSILON
        || frame.height <= f32::EPSILON
    {
        return;
    }
    commands.push(HostPaintCommand::text(
        frame,
        Some(clip.clone()),
        order,
        text.into_owned(),
        color,
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
