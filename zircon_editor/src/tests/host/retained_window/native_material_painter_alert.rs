use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use crate::ui::retained_host::{
    paint_template_nodes_for_test, TemplateNodeFrameData, TemplatePaneNodeData,
};
use zircon_runtime_interface::ui::style::{
    ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
};

const BACKGROUND: [u8; 4] = [0, 0, 0, 255];
const MUI_ALERT_WARNING: [u8; 4] = [237, 108, 2, 255];
const MUI_ALERT_ERROR: [u8; 4] = [211, 47, 47, 255];
const SLOT_DUPLICATE_RED: [u8; 4] = [201, 42, 33, 255];

#[test]
fn native_template_painter_draws_mui_alert_outlined_icon_message_action_geometry() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "WarningAlert".into(),
        node_id: "WarningAlert.node".into(),
        role: "Alert".into(),
        component_role: "alert".into(),
        component_variant: "outlined warning colorWarning hasIcon hasAction".into(),
        text: "Warning".into(),
        validation_level: "warning".into(),
        frame: frame(4.0, 4.0, 128.0, 34.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(140, 48, nodes);

    assert_eq!(pixel(&bytes, 140, 24, 4), MUI_ALERT_WARNING);
    assert_eq!(pixel(&bytes, 140, 25, 21), MUI_ALERT_WARNING);
    assert_eq!(pixel(&bytes, 140, 112, 21), MUI_ALERT_WARNING);
    assert_eq!(pixel(&bytes, 140, 90, 8), BACKGROUND);
}

#[test]
fn native_template_painter_draws_mui_alert_filled_close_action_and_consumes_slots() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "ErrorAlert".into(),
            node_id: "ErrorAlert.node".into(),
            role: "Alert".into(),
            component_role: "alert".into(),
            component_variant: "filled error colorError hasIcon hasCloseAction".into(),
            text: "Failed".into(),
            validation_level: "error".into(),
            frame: frame(4.0, 4.0, 128.0, 34.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "ErrorAlertSlot".into(),
            node_id: "ErrorAlert.slot".into(),
            role: "Icon".into(),
            component_role: "icon".into(),
            component_variant: "alertSlotIcon".into(),
            frame: frame(124.0, 4.0, 10.0, 10.0),
            button_style: resolved_background(SLOT_DUPLICATE_RED),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(140, 48, nodes);

    assert_eq!(pixel(&bytes, 140, 90, 20), MUI_ALERT_ERROR);
    assert_eq!(pixel(&bytes, 140, 126, 8), MUI_ALERT_ERROR);
    assert_ne!(pixel(&bytes, 140, 128, 8), SLOT_DUPLICATE_RED);
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

fn resolved_background(color: [u8; 4]) -> ResolvedButtonStyle {
    ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                color[0], color[1], color[2], color[3],
            ))),
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    }
}
