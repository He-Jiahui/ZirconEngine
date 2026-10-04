use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::identity::MESH_PROPERTY_ROW;

pub(super) fn component_property_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: MESH_PROPERTY_ROW.into(),
        role: "InputField".into(),
        component_role: "input-field".into(),
        text: "Visible".into(),
        value_text: "true".into(),
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
