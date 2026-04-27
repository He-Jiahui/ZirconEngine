use super::*;
use crate::ui::template_runtime::{UiComponentShowcaseDemoEventInput, SHOWCASE_DOCUMENT_ID};
use zircon_runtime::ui::component::{UiDragPayloadKind, UiValue};

impl SlintEditorHost {
    pub(super) fn dispatch_pane_surface_control_clicked(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) {
        self.focus_callback_source_window();
        if control_id == "ModulePluginAction" {
            self.dispatch_module_plugin_action(action_id);
            return;
        }
        let Some(result) = callback_dispatch::dispatch_builtin_pane_surface_control(
            &self.runtime,
            &self.pane_surface_bridge,
            control_id,
            UiEventKind::Click,
            vec![UiBindingValue::string(action_id)],
        ) else {
            self.set_status_line(format!("Unknown pane surface control {control_id}"));
            return;
        };

        self.apply_dispatch_result(result);
    }

    pub(super) fn dispatch_component_showcase_control_activated(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) {
        self.focus_callback_source_window();
        let input = demo_input_for_showcase_action(control_id, action_id);
        self.dispatch_component_showcase_event(control_id, action_id, input);
    }

    pub(super) fn dispatch_component_showcase_control_drag_delta(
        &mut self,
        control_id: &str,
        action_id: &str,
        delta: f64,
    ) {
        self.focus_callback_source_window();
        let input = if action_id.contains("LargeDragUpdate") {
            UiComponentShowcaseDemoEventInput::LargeDragDelta(delta)
        } else {
            UiComponentShowcaseDemoEventInput::DragDelta(delta)
        };
        self.dispatch_component_showcase_event(control_id, action_id, input);
    }

    pub(super) fn dispatch_component_showcase_control_edited(
        &mut self,
        control_id: &str,
        action_id: &str,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let input = demo_input_for_showcase_edit(action_id, value);
        self.dispatch_component_showcase_event(control_id, action_id, input);
    }

    pub(super) fn dispatch_component_showcase_option_selected(
        &mut self,
        control_id: &str,
        action_id: &str,
        option_id: &str,
    ) {
        self.focus_callback_source_window();
        self.dispatch_component_showcase_event(
            control_id,
            action_id,
            select_option(option_id, true),
        );
    }

    fn dispatch_component_showcase_event(
        &mut self,
        control_id: &str,
        action_id: &str,
        input: UiComponentShowcaseDemoEventInput,
    ) {
        let binding = self
            .component_showcase_runtime
            .project_document(SHOWCASE_DOCUMENT_ID)
            .ok()
            .and_then(|projection| {
                projection
                    .bindings
                    .into_iter()
                    .find(|binding| binding.binding_id == action_id)
            });
        let Some(binding) = binding else {
            self.set_status_line(format!("Unknown component showcase action {action_id}"));
            return;
        };

        match self
            .component_showcase_runtime
            .apply_showcase_demo_binding(&binding.binding, input)
        {
            Ok(()) => {
                self.set_status_line(format!("Showcase event dispatched: {control_id}"));
                self.presentation_dirty = true;
            }
            Err(error) => {
                self.set_status_line(format!("Showcase event failed: {error}"));
            }
        }
    }

    fn dispatch_module_plugin_action(&mut self, action_id: &str) {
        let Some((enabled, plugin_id)) = parse_module_plugin_action(action_id) else {
            self.set_status_line(format!("Unknown module plugin action {action_id}"));
            return;
        };
        let project_path = self.runtime.editor_snapshot().project_path;
        let result = crate::ui::workbench::project::project_root_path(&project_path)
            .map_err(|error| error.to_string())
            .and_then(|project_root| {
                let manifest_path = project_root.join("zircon-project.toml");
                let mut manifest =
                    zircon_runtime::asset::project::ProjectManifest::load(&manifest_path)
                        .map_err(|error| error.to_string())?;
                let report = self
                    .editor_manager
                    .set_native_aware_project_plugin_enabled(
                        &project_root,
                        &mut manifest,
                        plugin_id,
                        enabled,
                    )?;
                manifest
                    .save(&manifest_path)
                    .map_err(|error| error.to_string())?;
                Ok(report)
            });
        match result {
            Ok(report) => {
                let state = if report.enabled {
                    "enabled"
                } else {
                    "disabled"
                };
                self.set_status_line(format!("Plugin {} {state}", report.plugin_id));
                self.layout_dirty = true;
                self.presentation_dirty = true;
            }
            Err(error) => self.set_status_line(format!("Plugin action failed: {error}")),
        }
    }
}

fn demo_input_for_showcase_edit(action_id: &str, value: &str) -> UiComponentShowcaseDemoEventInput {
    let value = if action_id.contains("NumberField") || action_id.contains("RangeField") {
        value
            .parse::<f64>()
            .map(UiValue::Float)
            .unwrap_or_else(|_| UiValue::String(value.to_string()))
    } else {
        UiValue::String(value.to_string())
    };
    UiComponentShowcaseDemoEventInput::Value(value)
}

fn demo_input_for_showcase_action(
    control_id: &str,
    action_id: &str,
) -> UiComponentShowcaseDemoEventInput {
    match action_id {
        action if action.contains("NumberFieldDragUpdate") => {
            UiComponentShowcaseDemoEventInput::DragDelta(5.0)
        }
        action if action.contains("NumberFieldLargeDragUpdate") => {
            UiComponentShowcaseDemoEventInput::LargeDragDelta(1.0)
        }
        action if action.contains("NumberFieldChanged") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Float(47.0))
        }
        action if action.contains("RangeFieldChanged") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Float(72.0))
        }
        action if action.contains("InputField") => UiComponentShowcaseDemoEventInput::Value(
            UiValue::String("Runtime UI event".to_string()),
        ),
        action if action.contains("TextField") => UiComponentShowcaseDemoEventInput::Value(
            UiValue::String("Runtime UI event-driven text".to_string()),
        ),
        action if action.contains("ToggleButtonChanged") || action.contains("CheckboxChanged") => {
            UiComponentShowcaseDemoEventInput::Toggle(false)
        }
        action if action.contains("RadioChanged") => {
            UiComponentShowcaseDemoEventInput::Toggle(true)
        }
        action if action.contains("SegmentedControlChanged") => select_option("rotate", true),
        action if action.contains("DropdownChanged") => select_option("editor", true),
        action if action.contains("ComboBoxChanged") => select_option("native", true),
        action if action.contains("EnumFieldChanged") => select_option("UnityInspector", true),
        action if action.contains("FlagsFieldChanged") => select_option("Disabled", true),
        action if action.contains("SearchSelectChanged") => {
            select_option("runtime.ui.RangeField", true)
        }
        action if action.contains("ContextActionMenuChanged") => select_option("Open Source", true),
        action if action.contains("AssetFieldDropped") => {
            UiComponentShowcaseDemoEventInput::DropReference {
                kind: UiDragPayloadKind::Asset,
                reference: "res://materials/runtime_demo.mat".to_string(),
            }
        }
        action
            if action.contains("AssetFieldClear")
                || action.contains("AssetFieldLocate")
                || action.contains("AssetFieldOpen") =>
        {
            UiComponentShowcaseDemoEventInput::None
        }
        action if action.contains("InstanceFieldDropped") => {
            UiComponentShowcaseDemoEventInput::DropReference {
                kind: UiDragPayloadKind::SceneInstance,
                reference: "scene://Root/RuntimeDemoLight".to_string(),
            }
        }
        action if action.contains("ObjectFieldDropped") => {
            UiComponentShowcaseDemoEventInput::DropReference {
                kind: UiDragPayloadKind::Object,
                reference: "object://Selection/RuntimeDemo".to_string(),
            }
        }
        action if action.contains("GroupToggled") => {
            UiComponentShowcaseDemoEventInput::Toggle(false)
        }
        action if action.contains("FoldoutToggled") => {
            UiComponentShowcaseDemoEventInput::Toggle(true)
        }
        action if action.contains("TreeRowToggled") => {
            UiComponentShowcaseDemoEventInput::Toggle(false)
        }
        action if action.contains("ArrayFieldAddElement") => {
            UiComponentShowcaseDemoEventInput::AddElement {
                value: UiValue::String("MapField".to_string()),
            }
        }
        action if action.contains("ArrayFieldSetElement") => {
            UiComponentShowcaseDemoEventInput::SetElement {
                index: 1,
                value: UiValue::String("Vector3Field".to_string()),
            }
        }
        action if action.contains("ArrayFieldRemoveElement") => {
            UiComponentShowcaseDemoEventInput::RemoveElement { index: 0 }
        }
        action if action.contains("ArrayFieldMoveElement") => {
            UiComponentShowcaseDemoEventInput::MoveElement { from: 0, to: 1 }
        }
        action if action.contains("MapFieldAddEntry") => {
            UiComponentShowcaseDemoEventInput::AddMapEntry {
                key: "layer".to_string(),
                value: UiValue::String("Editor".to_string()),
            }
        }
        action if action.contains("MapFieldSetEntry") => {
            UiComponentShowcaseDemoEventInput::SetMapEntry {
                key: "speed".to_string(),
                value: UiValue::Float(2.5),
            }
        }
        action if action.contains("MapFieldRemoveEntry") => {
            UiComponentShowcaseDemoEventInput::RemoveMapEntry {
                key: "speed".to_string(),
            }
        }
        action if action.contains("ListRowClicked") => UiComponentShowcaseDemoEventInput::None,
        action if action.contains("Show") && control_id.starts_with("ComponentShowcase") => {
            UiComponentShowcaseDemoEventInput::None
        }
        _ => UiComponentShowcaseDemoEventInput::None,
    }
}

fn select_option(option_id: &str, selected: bool) -> UiComponentShowcaseDemoEventInput {
    UiComponentShowcaseDemoEventInput::SelectOption {
        option_id: option_id.to_string(),
        selected,
    }
}

fn parse_module_plugin_action(action_id: &str) -> Option<(bool, &str)> {
    action_id
        .strip_prefix("Plugin.Enable.")
        .map(|plugin_id| (true, plugin_id))
        .or_else(|| {
            action_id
                .strip_prefix("Plugin.Disable.")
                .map(|plugin_id| (false, plugin_id))
        })
}
