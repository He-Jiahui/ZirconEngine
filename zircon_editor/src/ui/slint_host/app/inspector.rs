use super::*;

impl SlintEditorHost {
    fn inspector_field_id(control_id: &str) -> Option<String> {
        if let Some(field_id) = control_id.strip_prefix("DynamicComponentField:") {
            return Some(field_id.to_string());
        }
        match control_id {
            "NameField" => Some("name".to_string()),
            "ParentField" => Some("parent".to_string()),
            "PositionXField" => Some("transform.translation.x".to_string()),
            "PositionYField" => Some("transform.translation.y".to_string()),
            "PositionZField" => Some("transform.translation.z".to_string()),
            _ => None,
        }
    }

    pub(super) fn inspector_reference_pointer_event(
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
        self.active_scene_drag_payload = None;
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
        let reference = format!("object://scene/node/{}", inspector.id);
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

    fn inspector_apply_arguments(&self) -> Result<Vec<UiBindingValue>, String> {
        let Some(inspector) = self.runtime.editor_snapshot().inspector else {
            return Err("Nothing selected".to_string());
        };

        let parent_value = if inspector.parent.trim().is_empty() {
            UiBindingValue::Null
        } else {
            UiBindingValue::string(inspector.parent.clone())
        };
        let mut changes = vec![
            UiBindingValue::array(vec![
                UiBindingValue::string("name"),
                UiBindingValue::string(inspector.name.clone()),
            ]),
            UiBindingValue::array(vec![UiBindingValue::string("parent"), parent_value]),
            UiBindingValue::array(vec![
                UiBindingValue::string("transform.translation.x"),
                UiBindingValue::string(inspector.translation[0].clone()),
            ]),
            UiBindingValue::array(vec![
                UiBindingValue::string("transform.translation.y"),
                UiBindingValue::string(inspector.translation[1].clone()),
            ]),
            UiBindingValue::array(vec![
                UiBindingValue::string("transform.translation.z"),
                UiBindingValue::string(inspector.translation[2].clone()),
            ]),
        ];
        changes.extend(
            inspector
                .plugin_components
                .iter()
                .filter(|component| component.drawer_available)
                .flat_map(|component| component.properties.iter())
                .filter(|property| property.editable)
                .map(|property| {
                    UiBindingValue::array(vec![
                        UiBindingValue::string(property.field_id.clone()),
                        UiBindingValue::string(property.value.clone()),
                    ])
                }),
        );
        let changes = UiBindingValue::array(changes);

        Ok(vec![UiBindingValue::string("entity://selected"), changes])
    }

    pub(super) fn dispatch_inspector_control_changed(&mut self, control_id: &str, value: &str) {
        let Some(field_id) = Self::inspector_field_id(control_id) else {
            self.set_status_line(format!("Unknown inspector change control {control_id}"));
            return;
        };

        self.focus_callback_source_window();
        let envelope = UiComponentEventEnvelope::new(
            "inspector.surface_controls",
            control_id,
            UiComponentBindingTarget::inspector("entity://selected", field_id.clone()),
            UiComponentEvent::ValueChanged {
                property: "value".to_string(),
                value: UiValue::String(value.to_string()),
            },
        )
        .with_component_id("InspectorField");

        match self.runtime.dispatch_ui_component_adapter_event(&envelope) {
            Ok(result) => {
                let refresh_presentation = result.refresh_projection || result.changed;
                self.set_status_line(
                    result
                        .status_text
                        .unwrap_or_else(|| format!("Inspector field updated: {field_id}")),
                );
                if refresh_presentation {
                    self.presentation_dirty = true;
                }
            }
            Err(error) => {
                self.set_status_line(format!("Inspector component binding failed: {error}"));
            }
        }
    }

    pub(super) fn dispatch_inspector_control_clicked(&mut self, control_id: &str) {
        let arguments = match control_id {
            "ApplyBatchButton" => match self.inspector_apply_arguments() {
                Ok(arguments) => arguments,
                Err(error) => {
                    self.set_status_line(error);
                    return;
                }
            },
            "DeleteSelected" => Vec::new(),
            _ => {
                self.set_status_line(format!("Unknown inspector click control {control_id}"));
                return;
            }
        };

        self.dispatch_inspector_surface_control(control_id, UiEventKind::Click, arguments);
    }

    pub(super) fn dispatch_inspector_surface_control(
        &mut self,
        control_id: &str,
        event_kind: UiEventKind,
        arguments: Vec<UiBindingValue>,
    ) {
        self.focus_callback_source_window();
        let Some(result) = callback_dispatch::dispatch_builtin_inspector_surface_control(
            &self.runtime,
            &self.inspector_surface_bridge,
            control_id,
            event_kind,
            arguments,
        ) else {
            self.set_status_line(format!("Unknown inspector surface control {control_id}"));
            return;
        };

        self.apply_dispatch_result(result);
    }
}
