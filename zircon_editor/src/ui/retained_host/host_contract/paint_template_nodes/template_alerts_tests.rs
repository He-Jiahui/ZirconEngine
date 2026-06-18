use super::super::super::data::TemplateNodeFrameData;
use super::super::super::paint_theme::PALETTE;
use super::super::style_selector::{
    WORKBENCH_ALERT_INFO_SURFACE, WORKBENCH_ALERT_WARNING_SURFACE, WORKBENCH_TOAST_ACTION,
    WORKBENCH_TOAST_BORDER, WORKBENCH_TOAST_SURFACE,
};
use super::super::template_nodes::paint_template_nodes_for_test;
use super::*;
use crate::ui::layouts::common::model_rc;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn workbench_alert_kind_matches_drawer_ids_and_toast_root() {
    assert_eq!(
        workbench_alert_kind(&alert_node("WorkbenchInfoAlert", "Info Alert", "info")),
        Some(WorkbenchAlertKind::Inline(AlertTone::Info))
    );
    assert_eq!(
        workbench_alert_kind(&alert_node("WorkbenchErrorAlert", "Error Alert", "error")),
        Some(WorkbenchAlertKind::Inline(AlertTone::Error))
    );
    assert_eq!(
        workbench_alert_kind(&alert_node(
            "WorkbenchToastRoot",
            "Operation completed successfully",
            "info"
        )),
        Some(WorkbenchAlertKind::Toast)
    );
    assert_eq!(
        workbench_alert_kind(&alert_node("PlainAlert", "Info Alert", "info")),
        None
    );
}

#[test]
fn workbench_info_alert_paints_tinted_surface_icon_and_label() {
    let bytes = paint_template_nodes_for_test(
        192,
        48,
        model_rc(vec![positioned_alert_node(
            "WorkbenchInfoAlert",
            "Info Alert",
            "info",
            8.0,
            8.0,
            160.0,
            32.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 192, 80, 24), WORKBENCH_ALERT_INFO_SURFACE);
    assert_eq!(pixel_at(&bytes, 192, 25, 24), PALETTE.info);
    assert!(changed_pixel_count(&bytes, 192, 38, 16, 62, 18) > 0);
    assert_eq!(pixel_at(&bytes, 192, 176, 24), [0, 0, 0, 255]);
}

#[test]
fn workbench_warning_alert_uses_warning_tone() {
    let bytes = paint_template_nodes_for_test(
        192,
        48,
        model_rc(vec![positioned_alert_node(
            "WorkbenchWarningAlert",
            "Warning Alert",
            "warning",
            8.0,
            8.0,
            160.0,
            32.0,
        )]),
    );

    assert_eq!(
        pixel_at(&bytes, 192, 150, 24),
        WORKBENCH_ALERT_WARNING_SURFACE
    );
    assert_eq!(pixel_at(&bytes, 192, 27, 18), PALETTE.warning);
    assert!(changed_pixel_count(&bytes, 192, 38, 16, 84, 18) > 0);
}

#[test]
fn workbench_toast_paints_status_mark_action_and_close() {
    let bytes = paint_template_nodes_for_test(
        320,
        48,
        model_rc(vec![positioned_alert_node(
            "WorkbenchToastRoot",
            "Operation completed successfully",
            "success",
            8.0,
            8.0,
            280.0,
            32.0,
        )]),
    );

    let surface_pixel = blend_over(WORKBENCH_TOAST_SURFACE, [0, 0, 0, 255]);
    assert_eq!(WORKBENCH_TOAST_SURFACE, [21, 48, 53, 247]);
    assert_eq!(WORKBENCH_TOAST_BORDER, [53, 199, 208, 20]);
    assert_eq!(pixel_at(&bytes, 320, 160, 12), surface_pixel);
    assert_eq!(
        pixel_at(&bytes, 320, 160, 8),
        blend_over(WORKBENCH_TOAST_BORDER, surface_pixel)
    );
    assert_ne!(pixel_at(&bytes, 320, 120, 24), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&bytes, 320, 35, 24), WORKBENCH_TOAST_ACTION);
    assert!(changed_pixel_count(&bytes, 320, 233, 16, 34, 18) > 0);
    assert!(changed_pixel_count(&bytes, 320, 269, 17, 12, 14) > 0);
}

#[test]
fn workbench_toast_uses_declared_status_mark_and_action_style() {
    let mut node = positioned_alert_node(
        "WorkbenchToastRoot",
        "Operation completed successfully",
        "success",
        8.0,
        8.0,
        280.0,
        32.0,
    );
    node.value_number = 12.0;
    node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(32, 159, 169);
    node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(35, 143, 152);

    let style = select_workbench_toast_style(&node);
    assert_eq!(toast_status_mark_size(&node), 12.0);
    assert_eq!(style.mark, [32, 159, 169, 255]);
    assert_eq!(style.action, [35, 143, 152, 255]);

    let bytes = paint_template_nodes_for_test(320, 48, model_rc(vec![node]));
    assert_eq!(pixel_at(&bytes, 320, 22, 20), [32, 159, 169, 255]);
}

#[test]
fn workbench_toast_style_uses_shared_state_priority() {
    let mut node = positioned_alert_node(
        "WorkbenchToastRoot",
        "Operation completed successfully",
        "success",
        8.0,
        8.0,
        280.0,
        32.0,
    );
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.disabled = true;

    let disabled = select_workbench_toast_style(&node);
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled.action, PALETTE.text_disabled);

    node.disabled = false;
    let pressed = select_workbench_toast_style(&node);
    assert_eq!(pressed.state, UiPainterResolvedState::Pressed);
    assert_eq!(pressed.border, PALETTE.focus_ring);

    node.pressed = false;
    let focused = select_workbench_toast_style(&node);
    assert_eq!(focused.state, UiPainterResolvedState::Focused);
}

#[test]
fn workbench_alert_style_uses_shared_state_priority() {
    let mut node = positioned_alert_node(
        "WorkbenchWarningAlert",
        "Warning Alert",
        "warning",
        8.0,
        8.0,
        160.0,
        32.0,
    );
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.disabled = true;

    let disabled = select_workbench_alert_style(&node, AlertTone::Warning);
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled.text, PALETTE.text_disabled);
    assert_eq!(disabled.surface, PALETTE.surface_disabled);

    node.disabled = false;
    let pressed = select_workbench_alert_style(&node, AlertTone::Warning);
    assert_eq!(pressed.state, UiPainterResolvedState::Pressed);
    assert_eq!(pressed.border, PALETTE.focus_ring);

    node.pressed = false;
    let focused = select_workbench_alert_style(&node, AlertTone::Warning);
    assert_eq!(focused.state, UiPainterResolvedState::Focused);
    assert_eq!(focused.border, PALETTE.focus_ring);
}

fn alert_node(control_id: &str, text: &str, tone: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Alert".into(),
        component_role: "alert".into(),
        text: text.into(),
        validation_level: tone.into(),
        icon_name: tone.into(),
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: 160.0,
            height: 32.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn positioned_alert_node(
    control_id: &str,
    text: &str,
    tone: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..alert_node(control_id, text, tone)
    }
}

fn changed_pixel_count(
    bytes: &[u8],
    frame_width: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * frame_width as usize) + px as usize) * 4;
            if bytes[index..index + 4] != [0, 0, 0, 255] {
                changed += 1;
            }
        }
    }
    changed
}

fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * frame_width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}

fn blend_over(source: [u8; 4], destination: [u8; 4]) -> [u8; 4] {
    let alpha = source[3] as u32;
    let inverse = 255 - alpha;
    [
        ((source[0] as u32 * alpha + destination[0] as u32 * inverse) / 255) as u8,
        ((source[1] as u32 * alpha + destination[1] as u32 * inverse) / 255) as u8,
        ((source[2] as u32 * alpha + destination[2] as u32 * inverse) / 255) as u8,
        255,
    ]
}
