// 图例上下限借用 generation 中的标签，运行时字宽只决定布局；拒绝无效帧后才为命令复制字符串。
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use crate::ui::weight_heatmap::WeightHeatmapGeneration;

use super::super::super::data::FrameRect;
use super::super::super::paint_text::measure_runtime_text_width;
use super::super::render_commands::HostPaintCommand;
use super::geometry::WeightHeatmapGeometry;
use super::palette::LEGEND_TEXT;

const LEGEND_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
const LEGEND_LINE_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;

#[cfg(test)]
#[path = "text/tests/capacity_tests.rs"]
mod capacity_tests;

pub(super) fn legend_label_width(generation: &WeightHeatmapGeneration) -> f32 {
    legend_label_width_from_labels(generation.high_label(), generation.low_label())
}

/// 此函数消费 geometry 为标签预留的右侧空间；不能由视觉层重新猜测图例宽度，调用方须先测量同一 generation。
pub(super) fn push_heatmap_legend_text(
    commands: &mut Vec<HostPaintCommand>,
    generation: &WeightHeatmapGeneration,
    geometry: &WeightHeatmapGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_label(
        commands,
        generation.high_label(),
        geometry.legend.y,
        geometry,
        clip,
        order + 5,
        opacity,
    );
    push_label(
        commands,
        generation.low_label(),
        geometry.legend.y + geometry.legend.height - LEGEND_LINE_HEIGHT,
        geometry,
        clip,
        order + 5,
        opacity,
    );
}

fn push_label(
    commands: &mut Vec<HostPaintCommand>,
    text: &str,
    y: f32,
    geometry: &WeightHeatmapGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if text.trim().is_empty() {
        return;
    }
    let frame = geometry.legend_label_frame(
        measure_runtime_text_width(&text, LEGEND_FONT_SIZE).ceil(),
        y,
        LEGEND_LINE_HEIGHT,
    );
    if frame.width <= f32::EPSILON || frame.height <= f32::EPSILON {
        return;
    }
    commands.push(HostPaintCommand::text(
        frame,
        Some(clip.clone()),
        order,
        text.to_owned(),
        LEGEND_TEXT,
        LEGEND_FONT_SIZE,
        LEGEND_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn legend_label_width_from_labels(high_label: &str, low_label: &str) -> f32 {
    [high_label, low_label]
        .into_iter()
        .filter(|label| !label.trim().is_empty())
        .map(|label| measure_runtime_text_width(label, LEGEND_FONT_SIZE).ceil())
        .fold(0.0, f32::max)
}

#[cfg(test)]
#[path = "tests/text.rs"]
mod tests;
