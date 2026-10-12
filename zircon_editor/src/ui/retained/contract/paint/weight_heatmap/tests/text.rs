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

    push_label(&mut commands, "High", 0.0, &geometry, &clip, 0, 1.0);

    assert!(commands.is_empty());
}
