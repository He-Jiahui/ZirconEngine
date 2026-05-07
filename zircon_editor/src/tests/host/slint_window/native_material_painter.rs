use std::rc::Rc;

use crate::ui::slint_host::{
    paint_template_nodes_for_test, TemplateNodeFrameData, TemplatePaneNodeData,
};
use slint::{ModelRc, VecModel};

const SURFACE_INSET: [u8; 4] = [23, 28, 36, 255];
const SURFACE_HOVER: [u8; 4] = [44, 53, 66, 255];
const SURFACE_PRESSED: [u8; 4] = [47, 64, 94, 255];
const SURFACE_SELECTED: [u8; 4] = [54, 83, 130, 255];
const SURFACE_DISABLED: [u8; 4] = [30, 33, 39, 255];
const ACCENT: [u8; 4] = [76, 125, 213, 255];
const FOCUS_RING: [u8; 4] = [146, 181, 255, 255];
const WARNING: [u8; 4] = [216, 162, 77, 255];
const WARNING_CONTAINER: [u8; 4] = [68, 51, 24, 255];
const ERROR: [u8; 4] = [217, 107, 99, 255];
const ERROR_CONTAINER: [u8; 4] = [74, 37, 41, 255];
const TEXT_DISABLED: [u8; 4] = [91, 99, 113, 255];

#[test]
fn native_template_painter_uses_material_state_palette_for_controls() {
    let nodes = model_rc(vec![
        material_node("Default", 4.0, 4.0),
        TemplatePaneNodeData {
            control_id: "Hovered".into(),
            node_id: "Hovered.node".into(),
            hovered: true,
            frame: frame(4.0, 32.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "Pressed".into(),
            node_id: "Pressed.node".into(),
            pressed: true,
            frame: frame(4.0, 60.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "Selected".into(),
            node_id: "Selected.node".into(),
            selected: true,
            frame: frame(4.0, 88.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "Disabled".into(),
            node_id: "Disabled.node".into(),
            disabled: true,
            frame: frame(4.0, 116.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "Primary".into(),
            node_id: "Primary.node".into(),
            button_variant: "primary".into(),
            frame: frame(4.0, 144.0),
            ..material_button_base()
        },
    ]);

    let bytes = paint_template_nodes_for_test(96, 176, nodes);

    assert_eq!(pixel(&bytes, 96, 8, 8), SURFACE_INSET);
    assert_eq!(pixel(&bytes, 96, 8, 36), SURFACE_HOVER);
    assert_eq!(pixel(&bytes, 96, 8, 64), SURFACE_PRESSED);
    assert_eq!(pixel(&bytes, 96, 8, 92), SURFACE_SELECTED);
    assert_eq!(pixel(&bytes, 96, 8, 120), SURFACE_DISABLED);
    assert_eq!(pixel(&bytes, 96, 8, 148), ACCENT);
    assert_eq!(pixel(&bytes, 96, 4, 88), FOCUS_RING);
}

#[test]
fn native_template_painter_uses_material_validation_and_text_state_palette() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "Warning".into(),
            node_id: "Warning.node".into(),
            validation_level: "warning".into(),
            text: "".into(),
            frame: frame(4.0, 4.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "Error".into(),
            node_id: "Error.node".into(),
            validation_level: "error".into(),
            text: "".into(),
            frame: frame(4.0, 32.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "DisabledText".into(),
            node_id: "DisabledText.node".into(),
            disabled: true,
            text: "Disabled".into(),
            frame: frame(4.0, 60.0),
            ..material_button_base()
        },
    ]);

    let bytes = paint_template_nodes_for_test(96, 88, nodes);

    assert_eq!(pixel(&bytes, 96, 8, 8), WARNING_CONTAINER);
    assert_eq!(pixel(&bytes, 96, 4, 4), WARNING);
    assert_eq!(pixel(&bytes, 96, 8, 36), ERROR_CONTAINER);
    assert_eq!(pixel(&bytes, 96, 4, 32), ERROR);
    assert!(
        region_contains_color(&bytes, 96, 8, 64, 56, 12, TEXT_DISABLED),
        "disabled text should use the Material disabled text color"
    );
}

fn material_node(control_id: &str, x: f32, y: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        node_id: format!("{control_id}.node").into(),
        frame: frame(x, y),
        ..material_button_base()
    }
}

fn material_button_base() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        role: "Button".into(),
        text: "".into(),
        surface_variant: "inset".into(),
        border_width: 1.0,
        corner_radius: 5.0,
        ..TemplatePaneNodeData::default()
    }
}

fn frame(x: f32, y: f32) -> TemplateNodeFrameData {
    TemplateNodeFrameData {
        x,
        y,
        width: 72.0,
        height: 20.0,
    }
}

fn pixel(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}

fn region_contains_color(
    bytes: &[u8],
    width: u32,
    x: u32,
    y: u32,
    region_width: u32,
    region_height: u32,
    expected: [u8; 4],
) -> bool {
    let y1 = y.saturating_add(region_height);
    let x1 = x.saturating_add(region_width);
    (y..y1).any(|row| (x..x1).any(|column| pixel(bytes, width, column, row) == expected))
}

fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}
