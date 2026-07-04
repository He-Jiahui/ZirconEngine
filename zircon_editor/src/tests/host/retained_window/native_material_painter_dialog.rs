use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use crate::ui::retained_host::{
    paint_template_nodes_for_test, TemplateNodeFrameData, TemplatePaneActionData,
    TemplatePaneNodeData,
};
use zircon_runtime_interface::ui::design_tokens::EditorPaletteTokens;

const BACKGROUND: [u8; 4] = [0, 0, 0, 255];
const DIALOG_SURFACE: [u8; 4] = EditorPaletteTokens::WORKBENCH_POPUP;
const DIALOG_ACTIVE_BORDER: [u8; 4] = EditorPaletteTokens::WORKBENCH_FOCUS_RING;
const DIALOG_ERROR: [u8; 4] = EditorPaletteTokens::WORKBENCH_ERROR;
const DIALOG_ERROR_BORDER: [u8; 4] = EditorPaletteTokens::WORKBENCH_ERROR_CONTAINER;
const DIALOG_DISABLED_TEXT: [u8; 4] = EditorPaletteTokens::WORKBENCH_TEXT_DISABLED;

#[test]
fn native_template_painter_draws_open_dialog_panel_text_and_action() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "DialogDemo".into(),
        node_id: "DialogDemo.node".into(),
        role: "Dialog".into(),
        component_role: "dialog".into(),
        text: "Scene Settings".into(),
        value_text: "Review scene-level settings before applying them.".into(),
        popup_open: true,
        actions: model_rc(vec![action("Apply", "dialog.apply")]),
        frame: frame(8.0, 8.0, 180.0, 112.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(200, 136, nodes);

    assert_eq!(pixel(&bytes, 200, 28, 28), DIALOG_SURFACE);
    assert_eq!(pixel(&bytes, 200, 90, 8), DIALOG_ACTIVE_BORDER);
    assert!(region_contains_non_color(
        &bytes,
        200,
        28,
        26,
        132,
        18,
        DIALOG_SURFACE
    ));
    assert!(region_contains_non_color(
        &bytes,
        200,
        28,
        56,
        152,
        18,
        DIALOG_SURFACE
    ));
    assert!(region_contains_non_color(
        &bytes,
        200,
        112,
        84,
        60,
        18,
        DIALOG_SURFACE
    ));
    assert_eq!(pixel(&bytes, 200, 196, 12), BACKGROUND);
}

#[test]
fn native_template_painter_draws_confirm_dialog_error_and_disabled_confirm_action() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "ConfirmDialogDemo".into(),
        node_id: "ConfirmDialogDemo.node".into(),
        role: "ConfirmDialog".into(),
        component_role: "confirm-dialog".into(),
        component_variant: "error colorError destructive confirmDisabled".into(),
        validation_level: "error".into(),
        text: "Delete selected prefab?".into(),
        value_text: "This removes the prefab reference from the scene.".into(),
        popup_open: true,
        actions: model_rc(vec![
            action("Cancel", "dialog.cancel"),
            action("Delete", "dialog.confirm"),
        ]),
        frame: frame(8.0, 8.0, 196.0, 118.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(220, 144, nodes);

    assert_eq!(pixel(&bytes, 220, 8, 64), DIALOG_ERROR);
    assert_eq!(pixel(&bytes, 220, 108, 8), DIALOG_ERROR_BORDER);
    assert!(region_contains_non_color(
        &bytes,
        220,
        28,
        26,
        152,
        18,
        DIALOG_SURFACE
    ));
    assert!(region_contains_non_color(
        &bytes,
        220,
        128,
        90,
        56,
        18,
        DIALOG_SURFACE
    ));
    assert!(region_contains_color(
        &bytes,
        220,
        128,
        90,
        56,
        18,
        DIALOG_DISABLED_TEXT
    ));
}

#[test]
fn native_template_painter_consumes_closed_dialog_without_surface_fallback() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "ClosedDialog".into(),
        node_id: "ClosedDialog.node".into(),
        role: "Dialog".into(),
        component_role: "dialog".into(),
        text: "Closed".into(),
        value_text: "Should not render".into(),
        popup_open: false,
        frame: frame(8.0, 8.0, 120.0, 80.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(144, 104, nodes);

    assert_eq!(changed_pixel_count(&bytes, BACKGROUND), 0);
}

fn model_rc<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(items)))
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> TemplateNodeFrameData {
    TemplateNodeFrameData {
        x,
        y,
        width,
        height,
    }
}

fn action(label: &str, action_id: &str) -> TemplatePaneActionData {
    TemplatePaneActionData {
        label: label.into(),
        action_id: action_id.into(),
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

fn changed_pixel_count(bytes: &[u8], background: [u8; 4]) -> usize {
    bytes
        .chunks_exact(4)
        .filter(|pixel| {
            pixel[0] != background[0]
                || pixel[1] != background[1]
                || pixel[2] != background[2]
                || pixel[3] != background[3]
        })
        .count()
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

fn region_contains_non_color(
    bytes: &[u8],
    width: u32,
    x: u32,
    y: u32,
    region_width: u32,
    region_height: u32,
    color: [u8; 4],
) -> bool {
    let y1 = y.saturating_add(region_height);
    let x1 = x.saturating_add(region_width);
    (y..y1).any(|row| (x..x1).any(|column| pixel(bytes, width, column, row) != color))
}
