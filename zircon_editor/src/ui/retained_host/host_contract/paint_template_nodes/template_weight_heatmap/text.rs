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

pub(super) fn legend_label_width(generation: &WeightHeatmapGeneration) -> f32 {
    legend_label_width_from_labels(generation.high_label(), generation.low_label())
}

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
        generation.high_label().to_owned(),
        geometry.legend.y,
        geometry,
        clip,
        order + 5,
        opacity,
    );
    push_label(
        commands,
        generation.low_label().to_owned(),
        geometry.legend.y + geometry.legend.height - LEGEND_LINE_HEIGHT,
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
    if frame.width <= f32::EPSILON || frame.height <= f32::EPSILON {
        return;
    }
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
    use super::{LEGEND_FONT_SIZE, legend_label_width_from_labels, push_label};
    use crate::ui::retained_host::host_contract::data::FrameRect;
    use crate::ui::retained_host::host_contract::paint_text::measure_runtime_text_width;
    use crate::ui::retained_host::host_contract::paint_template_nodes::template_weight_heatmap::geometry::WeightHeatmapGeometry;

    #[test]
    fn heatmap_legend_width_uses_runtime_text_measurement() {
        let high_label = "WWWWWW";
        let measured_width = legend_label_width_from_labels(high_label, "i");

        assert_eq!(
            measured_width,
            measure_runtime_text_width(high_label, LEGEND_FONT_SIZE).ceil()
        );
    }

    #[test]
    fn collapsed_heatmap_does_not_emit_legend_text() {
        let geometry = WeightHeatmapGeometry::from_frame(
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 32.0,
            },
            20.0,
        );
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
        };
        let mut commands = Vec::new();

        push_label(
            &mut commands,
            "High".to_string(),
            0.0,
            &geometry,
            &clip,
            0,
            1.0,
        );

        assert!(commands.is_empty());
    }
}
