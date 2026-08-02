use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_text::measure_runtime_text_width;
use super::super::render_commands::HostPaintCommand;
use super::geometry::WeightHeatmapGeometry;
use super::palette::LEGEND_TEXT;

const LEGEND_FONT_SIZE: f32 = 8.0;
const LEGEND_LINE_HEIGHT: f32 = 10.0;

pub(super) fn legend_label_width(node: &TemplatePaneNodeData) -> f32 {
    legend_label_width_from_labels(
        node.weight_heatmap.high_label.as_str(),
        node.weight_heatmap.low_label.as_str(),
    )
}

pub(super) fn push_heatmap_legend_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    geometry: &WeightHeatmapGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_label(
        commands,
        node.weight_heatmap.high_label.to_string(),
        geometry.legend.y - 1.0,
        geometry,
        clip,
        order + 5,
        opacity,
    );
    push_label(
        commands,
        node.weight_heatmap.low_label.to_string(),
        geometry.legend.y + geometry.legend.height - 11.0,
        geometry,
        clip,
        order + 5,
        opacity,
    );
}

fn push_label(
    commands: &mut Vec<HostPaintCommand>,
    text: String,
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
    commands.push(HostPaintCommand::text(
        frame,
        Some(clip.clone()),
        order,
        text,
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
mod tests {
    use super::{LEGEND_FONT_SIZE, legend_label_width_from_labels};
    use crate::ui::retained_host::host_contract::paint_text::measure_runtime_text_width;

    #[test]
    fn heatmap_legend_width_uses_runtime_text_measurement() {
        let high_label = "WWWWWW";
        let measured_width = legend_label_width_from_labels(high_label, "i");

        assert_eq!(
            measured_width,
            measure_runtime_text_width(high_label, LEGEND_FONT_SIZE).ceil()
        );
    }
}
