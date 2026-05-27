use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};

const MID_BACKGROUND: [u8; 4] = [100, 100, 100, 255];
const MUI_PAPER_ELEVATION3_SURFACE: [u8; 4] = [37, 37, 37, 255];
const MUI_PAPER_ELEVATION3_SHADOW_ON_MID: [u8; 4] = [68, 68, 68, 255];
const MUI_PAPER_DARK_BACKGROUND: [u8; 4] = [18, 18, 18, 255];
const MUI_PAPER_DARK_BORDER_ON_SURFACE: [u8; 4] = [46, 46, 46, 255];

#[test]
fn native_template_painter_draws_mui_paper_elevation_shadow_and_dark_overlay() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "Paper".into(),
        node_id: "Paper.node".into(),
        role: "Paper".into(),
        component_role: "paper".into(),
        component_variant: "elevation rounded".into(),
        elevation: 3.0,
        frame: frame(4.0, 4.0, 48.0, 24.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test_with_background(64, 44, MID_BACKGROUND, nodes);

    assert_eq!(pixel(&bytes, 64, 20, 16), MUI_PAPER_ELEVATION3_SURFACE);
    assert_eq!(
        pixel(&bytes, 64, 20, 30),
        MUI_PAPER_ELEVATION3_SHADOW_ON_MID
    );
    assert_eq!(pixel(&bytes, 64, 58, 30), MID_BACKGROUND);
}

#[test]
fn native_template_painter_draws_mui_paper_outlined_without_elevation_overlay() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "OutlinedPaper".into(),
        node_id: "OutlinedPaper.node".into(),
        role: "Paper".into(),
        component_role: "paper".into(),
        component_variant: "outlined rounded".into(),
        surface_variant: "paper-outlined".into(),
        elevation: 3.0,
        frame: frame(4.0, 4.0, 48.0, 24.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test_with_background(64, 44, MID_BACKGROUND, nodes);

    assert_eq!(pixel(&bytes, 64, 20, 16), MUI_PAPER_DARK_BACKGROUND);
    assert_eq!(pixel(&bytes, 64, 20, 4), MUI_PAPER_DARK_BORDER_ON_SURFACE);
    assert_eq!(pixel(&bytes, 64, 20, 30), MID_BACKGROUND);
}

fn model_rc(nodes: Vec<TemplatePaneNodeData>) -> ModelRc<TemplatePaneNodeData> {
    ModelRc::from(Rc::new(VecModel::from(nodes)))
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> TemplateNodeFrameData {
    TemplateNodeFrameData {
        x,
        y,
        width,
        height,
    }
}

fn pixel(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * width + x) * 4) as usize;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}
