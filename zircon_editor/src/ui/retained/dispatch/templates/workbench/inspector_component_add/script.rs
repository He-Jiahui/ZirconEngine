//! Script mode uses typed flags and names; only scalar properties use a JSON draft.

use super::*;
use crate::core::editing::command::{
    AddComponentScope, SetScriptBindingsPayload, SetScriptBindingsRequest,
    SET_SCRIPT_BINDINGS_OPERATION,
};
use crate::ui::host::ScriptPackageChoice;
use zircon_runtime::asset::SceneScriptBindingAsset;

const PACKAGE: &str = "WorkbenchComponentAddPackage";
const MODULE: &str = "WorkbenchComponentAddModule";

pub(super) struct ScriptBindingDraft {
    pub(super) before: Option<Vec<SceneScriptBindingAsset>>,
    pub(super) binding: SceneScriptBindingAsset,
    pub(super) index: Option<usize>,
    packages: Vec<ScriptPackageChoice>,
    page: usize,
    discovery_error: Option<String>,
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn open_component_add_script(
        &mut self,
        runtime: &EditorHostEventController,
        scope: AddComponentScope,
        before: Option<Vec<SceneScriptBindingAsset>>,
        index: Option<usize>,
    ) -> Result<(), String> {
        let previous_focus = self.surface().focus.focused;
        let catalog = runtime.inspector_component_add_catalog(0, Some(&scope))?;
        self.inspector_component_add = InspectorComponentAddState {
            catalog: Some(catalog),
            previous_focus,
            ..Default::default()
        };
        self.set_visible(PANEL, true).map_err(|e| e.to_string())?;
        self.seed_script_binding(runtime, before, index)?;
        self.publish_component_add_page()?;
        if let Some(node) = self
            .control_node_ids_with_descendants(MODULE)
            .first()
            .copied()
        {
            self.template_surface
                .surface
                .focus_node(node)
                .map_err(|e| e.to_string())?;
        }
        self.refresh_component_add()
    }

    pub(super) fn seed_script_binding(
        &mut self,
        runtime: &EditorHostEventController,
        before: Option<Vec<SceneScriptBindingAsset>>,
        index: Option<usize>,
    ) -> Result<(), String> {
        let (packages, discovery_error) = match runtime.inspector_script_binding_packages(true) {
            Ok(packages) => (packages, None),
            Err(error) => (Vec::new(), Some(error)),
        };
        let binding = match index {
            Some(index) => before
                .as_ref()
                .and_then(|bindings| bindings.get(index))
                .cloned()
                .ok_or("script binding index changed")?,
            None => SceneScriptBindingAsset {
                package: packages
                    .first()
                    .map(|p| p.package.clone())
                    .unwrap_or_default(),
                module: packages
                    .first()
                    .map(|p| p.module.clone())
                    .unwrap_or_default(),
                enabled: true,
                update: true,
                fixed_update: true,
                properties: Default::default(),
            },
        };
        self.inspector_component_add.draft =
            serde_json::to_string(&binding.properties).map_err(|e| e.to_string())?;
        self.inspector_component_add.selected = None;
        self.inspector_component_add.script = Some(ScriptBindingDraft {
            before,
            binding,
            index,
            packages,
            page: 0,
            discovery_error,
        });
        self.mutate_control_property(
            VALUES,
            "query",
            UiValue::String(self.inspector_component_add.draft.clone()),
        )
        .map_err(|e| e.to_string())?;
        self.publish_script_draft()?;
        let script = self.inspector_component_add.script.as_ref().unwrap();
        if let Some(error) = &script.discovery_error {
            self.component_add_feedback_text(format!(
                "{}: {error}",
                self.i18n
                    .translate("editor.inspector.script_binding.discovery_error")
            ))?;
        } else {
            let key = if script.packages.is_empty() {
                "editor.inspector.script_binding.no_packages"
            } else {
                "editor.inspector.script_binding.properties_hint"
            };
            self.component_add_feedback(key)?;
        }
        Ok(())
    }

    pub(super) fn publish_script_draft(&mut self) -> Result<(), String> {
        let script = self.inspector_component_add.script.as_ref();
        let visible = script.is_some();
        let editing = script.is_some_and(|script| script.index.is_some());
        let title_key = if visible {
            "editor.inspector.script_binding.add"
        } else {
            "editor.inspector.component_add.title"
        };
        let confirm_key = if editing {
            "editor.inspector.script_binding.apply"
        } else if visible {
            "editor.inspector.script_binding.attach"
        } else {
            "editor.inspector.component_add.confirm"
        };
        for (control, key) in [
            ("WorkbenchComponentAddTitle", title_key),
            ("WorkbenchComponentAddConfirm", confirm_key),
        ] {
            self.mutate_control_property(
                control,
                "text",
                UiValue::String(self.i18n.translate(key).to_string()),
            )
            .map_err(|e| e.to_string())?;
        }
        let script = self.inspector_component_add.script.as_ref();
        let options = script
            .map(|script| {
                script
                    .packages
                    .iter()
                    .enumerate()
                    .skip(script.page * PAGE_SIZE)
                    .take(PAGE_SIZE)
                    .map(|(index, package)| {
                        UiValue::String(format!(
                            "{index}|label={}",
                            super::super::inspector_script_bindings::option_label(&package.package)
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let selected = script.and_then(|script| {
            script
                .packages
                .iter()
                .position(|p| p.package == script.binding.package)
        });
        let package = script
            .map(|s| s.binding.package.clone())
            .unwrap_or_default();
        let module = script.map(|s| s.binding.module.clone()).unwrap_or_default();
        let package_disabled = script.is_none_or(|s| s.packages.is_empty());
        let keys = [
            (
                "WorkbenchComponentAddEnabled",
                if script.is_some_and(|s| s.binding.enabled) {
                    "enabled"
                } else {
                    "disabled"
                },
            ),
            (
                "WorkbenchComponentAddUpdate",
                if script.is_some_and(|s| s.binding.update) {
                    "update_on"
                } else {
                    "update_off"
                },
            ),
            (
                "WorkbenchComponentAddFixedUpdate",
                if script.is_some_and(|s| s.binding.fixed_update) {
                    "fixed_update_on"
                } else {
                    "fixed_update_off"
                },
            ),
        ];
        let texts: Vec<_> = keys
            .into_iter()
            .map(|(id, key)| {
                (
                    id,
                    self.i18n
                        .translate(&format!("editor.inspector.script_binding.{key}"))
                        .to_string(),
                )
            })
            .collect();
        for (id, property, value) in [
            (PACKAGE, "options", UiValue::Array(options)),
            (
                PACKAGE,
                "value",
                UiValue::String(selected.map(|i| i.to_string()).unwrap_or_default()),
            ),
            (PACKAGE, "value_text", UiValue::String(package)),
            (PACKAGE, "disabled", UiValue::Bool(package_disabled)),
            (MODULE, "query", UiValue::String(module)),
        ] {
            self.mutate_control_property(id, property, value)
                .map_err(|e| e.to_string())?;
        }
        for (id, text) in texts {
            self.mutate_control_property(id, "text", UiValue::String(text))
                .map_err(|e| e.to_string())?;
        }
        self.set_visible("WorkbenchComponentAddScriptFields", visible)
            .map_err(|e| e.to_string())?;
        for id in [
            "WorkbenchComponentAddRequiredLabel",
            "WorkbenchComponentAddRequired",
        ] {
            self.set_visible(id, !visible).map_err(|e| e.to_string())?;
        }
        let key = if visible {
            "editor.inspector.script_binding.properties"
        } else {
            "editor.inspector.component_add.values"
        };
        self.mutate_control_property(
            VALUES,
            "placeholder",
            UiValue::String(self.i18n.translate(key).to_string()),
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub(crate) fn select_script_package(
        &mut self,
        control: &str,
        option: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if control != PACKAGE {
            return None;
        }
        Some((|| {
            if self
                .select_dropdown_option(PACKAGE, option)
                .map_err(|e| e.to_string())?
            {
                let index = option.parse::<usize>().map_err(|e| e.to_string())?;
                if let Some(script) = &mut self.inspector_component_add.script {
                    if let Some(package) = script.packages.get(index) {
                        script.binding.package = package.package.clone();
                        script.binding.module = package.module.clone();
                    }
                }
                self.publish_component_add_page()?;
            }
            self.refresh_component_add()?;
            Ok(component_add_paint())
        })())
    }

    pub(super) fn dispatch_script_draft(
        &mut self,
        control: &str,
        action: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if !action.starts_with("workbench.inspector.component_add.script.") {
            return None;
        }
        Some((|| {
            if control == PACKAGE {
                self.toggle_popup(PACKAGE).map_err(|e| e.to_string())?;
            } else if let Some(script) = &mut self.inspector_component_add.script {
                match control {
                    "WorkbenchComponentAddEnabled" => {
                        script.binding.enabled = !script.binding.enabled
                    }
                    "WorkbenchComponentAddUpdate" => script.binding.update = !script.binding.update,
                    "WorkbenchComponentAddFixedUpdate" => {
                        script.binding.fixed_update = !script.binding.fixed_update
                    }
                    _ => {}
                }
                self.publish_script_draft()?;
            }
            self.refresh_component_add()?;
            Ok(component_add_paint())
        })())
    }

    pub(crate) fn edit_script_module(
        &mut self,
        control: &str,
        binding: &str,
        value: &str,
    ) -> Result<Option<bool>, BuiltinHostWindowTemplateBridgeError> {
        if !matches!(
            binding,
            "Inspector/ScriptBindingModuleEdit" | "Inspector/ScriptBindingModuleCommit"
        ) {
            return Ok(None);
        }
        if control != MODULE {
            return Ok(Some(false));
        }
        if let Some(script) = &mut self.inspector_component_add.script {
            script.binding.module = value.to_owned();
        }
        self.mutate_control_property(MODULE, "query", UiValue::String(value.to_owned()))?;
        let available = self
            .inspector_component_add
            .script
            .as_ref()
            .is_some_and(|script| {
                !script.binding.package.trim().is_empty()
                    && !script.binding.module.trim().is_empty()
            });
        self.mutate_control_property(
            "WorkbenchComponentAddConfirm",
            "disabled",
            UiValue::Bool(!available),
        )?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(Some(true))
    }

    pub(super) fn page_script_packages(&mut self, next: bool) -> Result<(), String> {
        if let Some(script) = &mut self.inspector_component_add.script {
            script.page = if next {
                (script.page + 1).min(script.packages.len().saturating_sub(1) / PAGE_SIZE)
            } else {
                script.page.saturating_sub(1)
            };
        }
        self.close_popup(PACKAGE).map_err(|e| e.to_string())?;
        self.publish_component_add_page()?;
        self.publish_script_draft()?;
        self.refresh_component_add()
    }

    pub(super) fn script_package_paging(&self) -> Option<(bool, bool, String)> {
        self.inspector_component_add.script.as_ref().map(|s| {
            (
                s.page > 0,
                (s.page + 1) * PAGE_SIZE < s.packages.len(),
                format!(
                    "{}/{}",
                    s.page + 1,
                    s.packages.len().div_ceil(PAGE_SIZE).max(1)
                ),
            )
        })
    }

    pub(super) fn confirm_script_binding(
        &mut self,
        runtime: &EditorHostEventController,
    ) -> Result<UiHostEventEffects, String> {
        let draft = self
            .inspector_component_add
            .script
            .as_ref()
            .ok_or("script draft is unavailable")?;
        // Legacy nested values remain intact when only names or flags are edited.
        let unchanged =
            serde_json::to_string(&draft.binding.properties).map_err(|e| e.to_string())?;
        let properties = match parse_properties(&self.inspector_component_add.draft) {
            _ if self.inspector_component_add.draft == unchanged => {
                draft.binding.properties.clone()
            }
            Ok(properties) => properties,
            Err(_) => {
                self.component_add_feedback("editor.inspector.script_binding.invalid_properties")?;
                self.refresh_component_add()?;
                return Ok(component_add_paint());
            }
        };
        let state = &self.inspector_component_add;
        let script = state.script.as_ref().ok_or("script draft is unavailable")?;
        let catalog = state
            .catalog
            .as_ref()
            .ok_or("script scope is unavailable")?;
        let mut binding = script.binding.clone();
        binding.properties = properties;
        let mut after = script.before.clone().unwrap_or_default();
        match script.index {
            Some(index) => {
                *after.get_mut(index).ok_or("script binding index changed")? = binding;
            }
            None => after.push(binding),
        }
        let request = SetScriptBindingsRequest {
            scope: catalog.scope.clone(),
            bindings: SetScriptBindingsPayload {
                node_id: catalog.scope.targets[0],
                before: script.before.clone(),
                after: Some(after),
            },
        };
        let invocation = EditorOperationInvocation::parse(SET_SCRIPT_BINDINGS_OPERATION)
            .map_err(|e| e.to_string())?
            .with_arguments(serde_json::to_value(request).map_err(|e| e.to_string())?);
        let mut effects = component_add_paint();
        match runtime.invoke_operation(EditorOperationSource::UiBinding, invocation) {
            Ok(record) => {
                apply_record_effects(&mut effects, &record);
                self.close_component_add()?;
            }
            Err(_) => {
                self.component_add_feedback("editor.inspector.script_binding.rejected")?;
                self.refresh_component_add()?;
            }
        }
        Ok(effects)
    }

    fn component_add_feedback_text(&mut self, text: String) -> Result<(), String> {
        self.mutate_control_property(
            "WorkbenchComponentAddFeedback",
            "text",
            UiValue::String(text),
        )
        .map_err(|e| e.to_string())
    }
}

fn parse_properties(
    text: &str,
) -> Result<std::collections::BTreeMap<String, serde_json::Value>, String> {
    let properties: std::collections::BTreeMap<String, serde_json::Value> =
        serde_json::from_str(text).map_err(|e| e.to_string())?;
    if properties.values().any(|value| {
        !matches!(
            value,
            serde_json::Value::Bool(_)
                | serde_json::Value::Number(_)
                | serde_json::Value::String(_)
        )
    }) {
        return Err("script properties must contain only strings, numbers or booleans".into());
    }
    Ok(properties)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn script_binding_properties_accept_scalars_and_reject_nested_or_nonobject_drafts() {
        assert_eq!(
            parse_properties(r#"{"role":"player","orb_total":8,"moving":true}"#)
                .unwrap()
                .len(),
            3
        );
        for text in [
            "[]",
            "null",
            "{",
            r#"{"nested":{}}"#,
            r#"{"array":[1]}"#,
            r#"{"value":null}"#,
        ] {
            assert!(parse_properties(text).is_err(), "{text}");
        }
    }
}
