use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::{
    TemplateNodeFrameData, TemplatePaneNodeData, TemplatePaneSampleGridData,
};
use crate::ui::sample_grid::{SampleGridGeneration, SampleGridGenerationInput, SampleGridPoint};

use super::super::super::template_nodes::paint_template_nodes_for_test;

pub(super) fn sample_grid_node(width: f32, height: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchSampleGrid".into(),
        role: "Canvas".into(),
        component_role: "canvas".into(),
        component_variant: "sample-grid".into(),
        sample_grid: TemplatePaneSampleGridData {
            generation: SampleGridGeneration::new(SampleGridGenerationInput {
                x_axis_label: "Direction (deg)".to_string(),
                y_axis_label: "Speed (cm/s)".to_string(),
                x_min: -180.0,
                x_max: 180.0,
                y_min: 0.0,
                y_max: 600.0,
                x_ticks: vec![-180.0, -90.0, 0.0, 90.0, 180.0],
                y_ticks: vec![0.0, 150.0, 300.0, 450.0, 600.0],
                points: vec![
                    SampleGridPoint::new(0.0, 600.0, "Run_Fwd", true),
                    SampleGridPoint::new(-180.0, 300.0, "Run_Left", false),
                ],
            }),
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

pub(super) fn paint_sample_grid(width: u32, height: u32) -> Vec<u8> {
    paint_template_nodes_for_test(
        width,
        height,
        model_rc(vec![sample_grid_node(width as f32, height as f32)]),
    )
}

pub(super) fn pixel_at(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}

pub(super) fn changed_pixels(bytes: &[u8], background: [u8; 4]) -> usize {
    bytes
        .chunks_exact(4)
        .filter(|pixel| *pixel != background.as_slice())
        .count()
}
