use super::super::RetainedEditorHost;
use zircon_runtime_interface::ui::component::{
    UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata,
};

fn scene_node_reference(id: u64) -> String {
    const PREFIX: &str = "object://scene/node/";
    let mut reference = String::with_capacity(PREFIX.len() + 20);
    reference.push_str(PREFIX);
    push_u64_decimal(&mut reference, id);
    reference
}

fn push_u64_decimal(output: &mut String, mut value: u64) {
    let mut digits = [0_u8; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    for digit in &digits[start..] {
        output.push(char::from(*digit));
    }
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn inspector_reference_pointer_event(
        &mut self,
        kind: i32,
        button: i32,
        _x: f32,
        _y: f32,
        _width: f32,
        _height: f32,
    ) {
        if button == 1 && kind == 2 {
            self.active_object_drag_payload = None;
            return;
        }
        if kind != 0 || button != 1 {
            return;
        }

        self.active_asset_drag_payload = None;
        self.retire_hierarchy_drag();
        self.focus_callback_source_window();
        self.active_object_drag_payload = self.object_drag_payload_from_selected_inspector();
        if let Some(summary) = self
            .active_object_drag_payload
            .as_ref()
            .and_then(UiDragPayload::source_summary)
        {
            self.set_status_line(format!("Object drag source: {summary}"));
        }
    }

    fn object_drag_payload_from_selected_inspector(&self) -> Option<UiDragPayload> {
        let inspector = self.runtime.editor_snapshot().inspector?;
        let reference = scene_node_reference(inspector.id);
        Some(
            UiDragPayload::new(UiDragPayloadKind::Object, reference.clone()).with_source(
                UiDragSourceMetadata {
                    source_surface: "inspector".to_string(),
                    source_control_id: "InspectorHeaderPanel".to_string(),
                    locator: Some(reference),
                    display_name: Some(inspector.name),
                    asset_kind: Some("Scene Object".to_string()),
                    ..UiDragSourceMetadata::default()
                },
            ),
        )
    }
}

#[cfg(test)]
#[path = "tests/drag_source_optimization_batch_fh_tests.rs"]
mod optimization_batch_fh_tests;
