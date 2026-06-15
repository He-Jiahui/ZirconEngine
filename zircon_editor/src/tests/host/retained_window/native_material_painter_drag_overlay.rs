use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use crate::ui::retained_host::{
    paint_template_nodes_for_test, TemplateNodeFrameData, TemplatePaneNodeData,
};

const BACKGROUND: [u8; 4] = [0, 0, 0, 255];
const PREVIEW_SURFACE_BLOCKED: [u8; 4] = [72, 32, 36, 255];
const PREVIEW_BORDER_BLOCKED: [u8; 4] = [239, 112, 102, 255];

#[test]
fn native_template_painter_draws_drag_overlay_preview_and_drop_indicator() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "DragOverlayDemo".into(),
        node_id: "DragOverlayDemo.node".into(),
        role: "DragOverlay".into(),
        component_role: "drag-overlay".into(),
        popup_open: true,
        dragging: true,
        drop_hovered: true,
        active_drag_target: true,
        drag_payload_kind: "asset".into(),
        drag_payload_label: "StoneWall.mesh".into(),
        drag_payload_reference: "assets/stone_wall.mesh".into(),
        has_drag_cursor: true,
        drag_cursor_x: 72.0,
        drag_cursor_y: 48.0,
        drag_offset_x: 16.0,
        drag_offset_y: 18.0,
        drag_preview_width: 184.0,
        drag_preview_height: 36.0,
        drop_allowed: false,
        has_drop_target: true,
        drop_target_x: 24.0,
        drop_target_y: 148.0,
        drop_target_width: 280.0,
        drop_target_height: 30.0,
        drop_indicator_edge: "bottom".into(),
        frame: frame(0.0, 0.0, 360.0, 220.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(360, 220, nodes);

    assert_eq!(pixel(&bytes, 360, 92, 80), PREVIEW_SURFACE_BLOCKED);
    assert_eq!(pixel(&bytes, 360, 100, 66), PREVIEW_BORDER_BLOCKED);
    assert_eq!(pixel(&bytes, 360, 108, 84), PREVIEW_BORDER_BLOCKED);
    assert_eq!(pixel(&bytes, 360, 80, 176), PREVIEW_BORDER_BLOCKED);
    assert_eq!(pixel(&bytes, 360, 4, 4), BACKGROUND);
}

#[test]
fn native_template_painter_consumes_closed_drag_overlay_without_surface_fallback() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "ClosedDragOverlay".into(),
        node_id: "ClosedDragOverlay.node".into(),
        role: "DragOverlay".into(),
        component_role: "drag-overlay".into(),
        popup_open: false,
        dragging: false,
        text: "Should not render".into(),
        frame: frame(0.0, 0.0, 200.0, 80.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(240, 120, nodes);

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
