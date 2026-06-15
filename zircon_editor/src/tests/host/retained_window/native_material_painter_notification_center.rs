use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use crate::ui::retained_host::{
    paint_template_nodes_for_test, TemplateNodeFrameData, TemplatePaneNodeData,
    TemplatePaneOptionData,
};

const BACKGROUND: [u8; 4] = [0, 0, 0, 255];
const PANEL_SURFACE: [u8; 4] = [17, 24, 29, 255];
const PANEL_BORDER: [u8; 4] = [45, 58, 66, 255];
const ROW_UNREAD_SURFACE: [u8; 4] = [21, 48, 53, 255];
const ROW_FOCUSED_SURFACE: [u8; 4] = [24, 58, 63, 255];
const ACCENT: [u8; 4] = [53, 199, 208, 255];
const ERROR: [u8; 4] = [239, 112, 102, 255];
const SUCCESS: [u8; 4] = [66, 184, 131, 255];

#[test]
fn native_template_painter_draws_notification_center_panel_and_rows() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "NotificationCenterDemo".into(),
        node_id: "NotificationCenterDemo.node".into(),
        role: "NotificationCenter".into(),
        component_role: "notification-center".into(),
        popup_open: true,
        text: "Notifications".into(),
        structured_options: model_rc(vec![
            notification(
                "build",
                "Build failed",
                "Shader compile error",
                "error",
                true,
                true,
                false,
            ),
            notification(
                "asset",
                "Asset import complete",
                "StoneWall.mesh ready",
                "success",
                false,
                true,
                true,
            ),
        ]),
        frame: frame(8.0, 8.0, 220.0, 156.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(248, 184, nodes);

    assert_eq!(pixel(&bytes, 248, 18, 30), PANEL_SURFACE);
    assert_eq!(pixel(&bytes, 248, 96, 8), PANEL_BORDER);
    assert_eq!(pixel(&bytes, 248, 96, 44), ACCENT);
    assert_eq!(pixel(&bytes, 248, 160, 66), ROW_UNREAD_SURFACE);
    assert_eq!(pixel(&bytes, 248, 27, 56), ERROR);
    assert_eq!(pixel(&bytes, 248, 160, 120), ROW_FOCUSED_SURFACE);
    assert_eq!(pixel(&bytes, 248, 27, 110), SUCCESS);
    assert_eq!(pixel(&bytes, 248, 244, 12), BACKGROUND);
}

#[test]
fn native_template_painter_consumes_closed_notification_center_without_surface_fallback() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "ClosedNotificationCenter".into(),
        node_id: "ClosedNotificationCenter.node".into(),
        role: "NotificationCenter".into(),
        component_role: "notification-center".into(),
        popup_open: false,
        text: "Notifications".into(),
        frame: frame(8.0, 8.0, 220.0, 156.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(248, 184, nodes);

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

fn notification(
    id: &str,
    label: &str,
    description: &str,
    tone: &str,
    selected: bool,
    unread: bool,
    focused: bool,
) -> TemplatePaneOptionData {
    TemplatePaneOptionData {
        id: id.into(),
        label: label.into(),
        description: description.into(),
        tone: tone.into(),
        selected,
        unread,
        special: unread,
        focused,
        ..TemplatePaneOptionData::default()
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
