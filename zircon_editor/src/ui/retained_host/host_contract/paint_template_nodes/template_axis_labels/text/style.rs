//! 轴文字命令从轴 palette 和字体密度组装样式；这里使用普通文本排版，不把标签当成可编辑字段。

use super::super::super::super::data::TemplatePaneNodeData;
use super::super::metrics::AxisLabelMetrics;
use super::super::style::axis_label_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) struct AxisLabelTextCommandStyle {
    pub color: [u8; 4],
    pub font_size: f32,
    pub line_height: f32,
    pub paint_style: UiTextRunPaintStyle,
}

pub(super) fn axis_label_text_command_style(
    node: &TemplatePaneNodeData,
    metrics: &AxisLabelMetrics,
) -> AxisLabelTextCommandStyle {
    AxisLabelTextCommandStyle {
        color: axis_label_color(node),
        // TODO: [CR-EDITOR-PAINT-ROWS-0005] 变换轴 ZUI 为节点声明 caption font_size，
        // 但专用命令固定用宿主 body+border 指标。需确认这是有意的视觉覆盖；
        // 若声明字号应生效，需按节点字号及行高约束重新投影，而非只改颜色。
        font_size: metrics.font_size,
        line_height: metrics.line_height,
        paint_style: axis_label_text_paint_style(),
    }
}

fn axis_label_text_paint_style() -> UiTextRunPaintStyle {
    UiTextRunPaintStyle::default()
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
