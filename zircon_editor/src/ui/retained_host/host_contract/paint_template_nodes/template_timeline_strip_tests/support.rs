use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::{
    TemplateNodeFrameData, TemplatePaneNodeData, TemplatePaneTimelineKeyData,
    TemplatePaneTimelineStripData,
};

use super::super::super::template_nodes::paint_template_nodes_for_test;

pub(super) fn timeline_strip_node(width: f32, height: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchTimelineStrip".into(),
        role: "Canvas".into(),
        component_role: "canvas".into(),
        component_variant: "timeline-strip".into(),
        timeline_strip: TemplatePaneTimelineStripData {
            duration: 3.0,
            current_time: 2.25,
            tick_interval: 0.5,
            track_label: "Run_Fwd".into(),
            keys: model_rc(vec![
                TemplatePaneTimelineKeyData {
                    time: 0.0,
                    label: "Start".into(),
                    selected: false,
                },
                TemplatePaneTimelineKeyData {
                    time: 2.0,
                    label: "Run_Fwd".into(),
                    selected: true,
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

pub(super) fn paint_timeline_strip(width: u32, height: u32) -> Vec<u8> {
    paint_template_nodes_for_test(
        width,
        height,
        model_rc(vec![timeline_strip_node(width as f32, height as f32)]),
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
