//! Paged Scripts section. Edit shares the Add Component form and typed operation.

use crate::core::editing::command::{
    SetScriptBindingsPayload, SetScriptBindingsRequest, SET_SCRIPT_BINDINGS_OPERATION,
};
use crate::core::editor_operation::{EditorOperationInvocation, EditorOperationSource};
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::{apply_record_effects, UiHostEventEffects};
use crate::ui::workbench::snapshot::{InspectorScriptBindingSnapshot, InspectorSnapshot};
use zircon_runtime::asset::SceneScriptBindingAsset;
use zircon_runtime::scene::NodeId;
use zircon_runtime_interface::ui::component::UiValue;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const SELECT: &str = "WorkbenchScriptBindingSelect";
const PAGE_SIZE: usize = 16;

#[derive(Default)]
pub(super) struct InspectorScriptsState {
    node: Option<NodeId>,
    bindings: Vec<InspectorScriptBindingSnapshot>,
    error: Option<String>,
    selected: usize,
}

pub(super) fn option_label(value: &str) -> String {
    // The retained option wire format uses these characters as delimiters.
    value.replace('|', "∣").replace(',', "，")
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn sync_inspector_scripts(
        &mut self,
        inspector: Option<&InspectorSnapshot>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let state = &mut self.inspector_scripts;
        if state.node != inspector.map(|inspector| inspector.id) {
            state.selected = 0;
        }
        state.node = inspector.map(|inspector| inspector.id);
        let result = inspector
            .map(|inspector| inspector.script_bindings.clone())
            .unwrap_or_else(|| Ok(Vec::new()));
        match result {
            Ok(bindings) => {
                state.bindings = bindings;
                state.error = None;
            }
            Err(error) => {
                state.bindings.clear();
                state.error = Some(error);
            }
        }
        state.selected = state.selected.min(state.bindings.len().saturating_sub(1));
        self.publish_inspector_scripts()
    }

    fn publish_inspector_scripts(&mut self) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let state = &self.inspector_scripts;
        let selected = state.bindings.get(state.selected);
        let page = state.selected / PAGE_SIZE;
        let options = state
            .bindings
            .iter()
            .enumerate()
            .skip(page * PAGE_SIZE)
            .take(PAGE_SIZE)
            .map(|(index, binding)| {
                UiValue::String(format!(
                    "{index}|label={}",
                    option_label(&format!("{} · {}", binding.package, binding.module))
                ))
            })
            .collect();
        let label = selected
            .map(|binding| format!("{} · {}", binding.package, binding.module))
            .unwrap_or_default();
        let status = match &state.error {
            Some(error) => format!(
                "{}: {error}",
                self.i18n
                    .translate("editor.inspector.script_binding.unavailable")
            ),
            None if selected.is_none() => self
                .i18n
                .translate("editor.inspector.script_binding.empty")
                .to_string(),
            None => format!("{} / {}", state.selected + 1, state.bindings.len()),
        };
        let enabled_key = if selected.is_some_and(|binding| binding.enabled) {
            "editor.inspector.script_binding.enabled"
        } else {
            "editor.inspector.script_binding.disabled"
        };
        let enabled_text = self.i18n.translate(enabled_key).to_string();
        let disabled = selected.is_none();
        let previous = page == 0;
        let next = (page + 1) * PAGE_SIZE >= state.bindings.len();
        let index = state.selected.to_string();
        let visible = state.node.is_some();
        for (control, property, value) in [
            (SELECT, "options", UiValue::Array(options)),
            (SELECT, "value", UiValue::String(index)),
            (SELECT, "value_text", UiValue::String(label)),
            (SELECT, "disabled", UiValue::Bool(disabled)),
            (
                "WorkbenchScriptBindingStatus",
                "text",
                UiValue::String(status),
            ),
            (
                "WorkbenchScriptBindingEnable",
                "text",
                UiValue::String(enabled_text),
            ),
            (
                "WorkbenchScriptBindingEnable",
                "disabled",
                UiValue::Bool(disabled),
            ),
            (
                "WorkbenchScriptBindingEdit",
                "disabled",
                UiValue::Bool(disabled),
            ),
            (
                "WorkbenchScriptBindingRemove",
                "disabled",
                UiValue::Bool(disabled),
            ),
            (
                "WorkbenchScriptBindingPrevious",
                "disabled",
                UiValue::Bool(previous),
            ),
            (
                "WorkbenchScriptBindingNext",
                "disabled",
                UiValue::Bool(next),
            ),
        ] {
            self.mutate_control_property(control, property, value)?;
        }
        self.set_visible("WorkbenchInspectorScripts", visible)?;
        Ok(())
    }

    pub(crate) fn select_inspector_script_binding(
        &mut self,
        control: &str,
        option: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if control != SELECT {
            return None;
        }
        Some((|| {
            if self
                .select_dropdown_option(SELECT, option)
                .map_err(|e| e.to_string())?
            {
                let index = option.parse::<usize>().map_err(|e| e.to_string())?;
                if index < self.inspector_scripts.bindings.len() {
                    self.inspector_scripts.selected = index;
                }
                self.publish_inspector_scripts()
                    .map_err(|e| e.to_string())?;
            }
            self.refresh_script_controls()
        })())
    }

    pub(crate) fn dispatch_inspector_scripts(
        &mut self,
        runtime: &EditorHostEventController,
        control: &str,
        action: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if !action.starts_with("workbench.inspector.script_binding.") {
            return None;
        }
        Some((|| {
            match (control, action.rsplit('.').next().unwrap_or_default()) {
                (SELECT, "select") => self.toggle_popup(SELECT).map_err(|e| e.to_string())?,
                ("WorkbenchScriptBindingPrevious", "previous") => {
                    self.inspector_scripts.selected =
                        self.inspector_scripts.selected.saturating_sub(PAGE_SIZE);
                    self.publish_inspector_scripts()
                        .map_err(|e| e.to_string())?;
                }
                ("WorkbenchScriptBindingNext", "next") => {
                    self.inspector_scripts.selected = (self.inspector_scripts.selected + PAGE_SIZE)
                        .min(self.inspector_scripts.bindings.len().saturating_sub(1));
                    self.publish_inspector_scripts()
                        .map_err(|e| e.to_string())?;
                }
                ("WorkbenchScriptBindingEdit", "edit") => {
                    return self.open_script_binding_edit(runtime);
                }
                ("WorkbenchScriptBindingEnable", "enable")
                | ("WorkbenchScriptBindingRemove", "remove") => {
                    return self
                        .mutate_inspector_script_binding(runtime, action.ends_with(".remove"));
                }
                _ => {}
            }
            self.refresh_script_controls()
        })())
    }

    fn script_binding_request(
        &self,
        runtime: &EditorHostEventController,
        after: Option<Vec<SceneScriptBindingAsset>>,
    ) -> Result<SetScriptBindingsRequest, String> {
        let node_id = self
            .inspector_scripts
            .node
            .ok_or("script binding entity is unavailable")?;
        let before: Vec<_> = self
            .inspector_scripts
            .bindings
            .iter()
            .map(SceneScriptBindingAsset::from)
            .collect();
        let before = (!before.is_empty()).then_some(before);
        let scope = runtime.inspector_script_binding_scope(node_id, &before)?;
        Ok(SetScriptBindingsRequest {
            scope,
            bindings: SetScriptBindingsPayload {
                node_id,
                before,
                after,
            },
        })
    }

    fn open_script_binding_edit(
        &mut self,
        runtime: &EditorHostEventController,
    ) -> Result<UiHostEventEffects, String> {
        let request = self.script_binding_request(runtime, None)?;
        let selected = self.inspector_scripts.selected;
        self.open_component_add_script(
            runtime,
            request.scope,
            request.bindings.before,
            Some(selected),
        )?;
        self.refresh_script_controls()
    }

    fn mutate_inspector_script_binding(
        &mut self,
        runtime: &EditorHostEventController,
        remove: bool,
    ) -> Result<UiHostEventEffects, String> {
        let mut bindings: Vec<_> = self
            .inspector_scripts
            .bindings
            .iter()
            .map(SceneScriptBindingAsset::from)
            .collect();
        let index = self.inspector_scripts.selected;
        let Some(binding) = bindings.get_mut(index) else {
            return self.refresh_script_controls();
        };
        if remove {
            bindings.remove(index);
        } else {
            binding.enabled = !binding.enabled;
        }
        let after = (!bindings.is_empty()).then_some(bindings);
        let request = self.script_binding_request(runtime, after)?;
        let invocation = EditorOperationInvocation::parse(SET_SCRIPT_BINDINGS_OPERATION)
            .map_err(|e| e.to_string())?
            .with_arguments(serde_json::to_value(request).map_err(|e| e.to_string())?);
        let record = runtime
            .invoke_operation(EditorOperationSource::UiBinding, invocation)
            .map_err(|e| e.to_string())?;
        let mut effects = self.refresh_script_controls()?;
        apply_record_effects(&mut effects, &record);
        Ok(effects)
    }

    fn refresh_script_controls(&mut self) -> Result<UiHostEventEffects, String> {
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())
            .map_err(|e| e.to_string())?;
        let mut effects = UiHostEventEffects::default();
        effects.request_paint_only();
        Ok(effects)
    }
}
