use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::{
    TemplateNodeFrameData, TemplatePaneNodeData, TemplatePaneWeightHeatmapData,
    TemplatePaneWeightHeatmapSourceData,
};

use super::super::super::template_nodes::paint_template_nodes_for_test;

fn weight_heatmap_node(width: f32, height: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchWeightHeatmap".into(),
        role: "Canvas".into(),
        component_role: "canvas".into(),
        component_variant: "weight-heatmap".into(),
        weight_heatmap: TemplatePaneWeightHeatmapData {
            columns: 16,
            rows: 10,
            low_label: "0.0".into(),
            high_label: "1.0".into(),
            sources: model_rc(vec![
                TemplatePaneWeightHeatmapSourceData {
                    x: 0.5,
                    y: 0.58,
                    weight: 1.0,
                    selected: true,
                },
                TemplatePaneWeightHeatmapSourceData {
                    x: 0.15,
                    y: 0.2,
                    weight: 0.35,
                    selected: false,
                },
                TemplatePaneWeightHeatmapSourceData {
                    x: 0.85,
                    y: 0.25,
                    weight: 0.45,
                    selected: false,
                },
            ]),
        },
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn paint_weight_heatmap(width: u32, height: u32) -> Vec<u8> {
    paint_template_nodes_for_test(
        width,
        height,
        model_rc(vec![weight_heatmap_node(width as f32, height as f32)]),
    )
}

pub(super) fn changed_pixels(bytes: &[u8], background: [u8; 4]) -> usize {
    bytes
        .chunks_exact(4)
        .filter(|pixel| *pixel != background.as_slice())
        .count()
}
