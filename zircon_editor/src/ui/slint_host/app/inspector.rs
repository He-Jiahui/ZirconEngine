use super::*;

impl SlintEditorHost {
    fn inspector_field_id(control_id: &str) -> Option<&'static str> {
        match control_id {
            "NameField" => Some("name"),
            "ParentField" => Some("parent"),
            "PositionXField" => Some("transform.translation.x"),
            "PositionYField" => Some("transform.translation.y"),
            "PositionZField" => Some("transform.translation.z"),
            _ => None,
        }
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
        let changes = UiBindingValue::array(vec![
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
        ]);

        Ok(vec![UiBindingValue::string("entity://selected"), changes])
    }

    pub(super) fn dispatch_inspector_control_changed(&mut self, control_id: &str, value: &str) {
        let Some(field_id) = Self::inspector_field_id(control_id) else {
            self.set_status_line(format!("Unknown inspector change control {control_id}"));
            return;
        };

        self.dispatch_inspector_surface_control(
            control_id,
            UiEventKind::Change,
            vec![
                UiBindingValue::string("entity://selected"),
                UiBindingValue::string(field_id),
                UiBindingValue::string(value),
            ],
        );
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
