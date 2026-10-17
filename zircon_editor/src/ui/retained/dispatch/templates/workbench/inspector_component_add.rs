//! A 32-entry metadata page and one selected schema/default payload. The details
//! picker uses the existing pane scroll owner. Names/JSON remain data.

use crate::core::editing::command::{AddComponentCatalog, AddComponentRequest, ADD_COMPONENT_OPERATION};
use crate::core::editor_operation::{EditorOperationInvocation, EditorOperationSource};
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::{apply_record_effects, UiHostEventEffects};
use zircon_runtime::scene::world::DynamicComponentCreationTemplate;
use zircon_runtime_interface::ui::component::UiValue;
use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const PANEL: &str = "WorkbenchComponentAddPanel";
const TYPE: &str = "WorkbenchComponentAddType";
const VALUES: &str = "WorkbenchComponentAddValues";
const PAGE_SIZE: usize = 32;

#[derive(Default)]
pub(super) struct InspectorComponentAddState {
    catalog: Option<AddComponentCatalog>,
    selected: Option<DynamicComponentCreationTemplate>,
    draft: String,
    previous_focus: Option<UiNodeId>,
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn dispatch_inspector_component_add(
        &mut self, runtime: &EditorHostEventController, control: &str, action: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        let result = match (control, action) {
            ("WorkbenchAddComponent", "workbench.inspector.component.add") => self.open_component_add(runtime),
            ("WorkbenchComponentAddConfirm", "workbench.inspector.component_add.confirm") => {
                return Some(self.confirm_component_add(runtime));
            }
            ("WorkbenchComponentAddCancel", "workbench.inspector.component_add.cancel") => self.close_component_add(),
            ("WorkbenchComponentAddPrevious", "workbench.inspector.component_add.previous") => self.page_component_add(runtime, false),
            ("WorkbenchComponentAddNext", "workbench.inspector.component_add.next") => self.page_component_add(runtime, true),
            (TYPE, "workbench.inspector.component_add.type.open") => self.toggle_popup(TYPE)
                .map_err(|error| error.to_string()).and_then(|()| self.refresh_component_add()),
            _ => return None,
        };
        Some(result.map(|()| component_add_paint()))
    }

    fn open_component_add(&mut self, runtime: &EditorHostEventController) -> Result<(), String> {
        let previous_focus = self.surface().focus.focused.or_else(|| {
            self.control_node_ids_with_descendants("WorkbenchAddComponent").first().copied()
        });
        self.inspector_component_add = InspectorComponentAddState { previous_focus, ..Default::default() };
        self.set_visible(PANEL, true).map_err(|error| error.to_string())?;
        match runtime.inspector_component_add_catalog(0, None) {
            Ok(catalog) => {
                self.inspector_component_add.catalog = Some(catalog);
                self.seed_first_component_add(runtime)?;
            }
            Err(_) => {
                self.seed_component_add_draft()?;
                self.component_add_feedback("editor.inspector.component_add.unavailable")?;
            }
        }
        self.publish_component_add_page()?;
        self.refresh_component_add()?;
        let control = if self.inspector_component_add.selected.is_some() { TYPE } else { "WorkbenchComponentAddCancel" };
        if let Some(node) = self.control_node_ids_with_descendants(control).first().copied() {
            self.template_surface.surface.focus_node(node).map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    fn seed_first_component_add(&mut self, runtime: &EditorHostEventController) -> Result<(), String> {
        let selected = self.inspector_component_add.catalog.as_ref().and_then(|catalog| {
            catalog.entries.iter().find(|entry| !entry.already_present)
                .map(|entry| (catalog.scope.clone(), entry.type_path.clone()))
        });
        let (selected, feedback) = match selected {
            Some((scope, path)) => match runtime.inspector_component_add_template(&scope, &path) {
                Ok(selected) => (Some(selected), "editor.inspector.component_add.initial_values"),
                Err(_) => (None, "editor.inspector.component_add.unavailable"),
            },
            None => (None, "editor.inspector.component_add.empty"),
        };
        self.inspector_component_add.selected = selected;
        self.seed_component_add_draft()?;
        self.component_add_feedback(feedback)
    }

    fn seed_component_add_draft(&mut self) -> Result<(), String> {
        self.inspector_component_add.draft = match &self.inspector_component_add.selected {
            Some(selected) => serde_json::to_string(&selected.spec.value).map_err(|error| error.to_string())?,
            None => String::new(),
        };
        self.mutate_control_property(VALUES, "query", UiValue::String(self.inspector_component_add.draft.clone()))
            .map_err(|error| error.to_string())
    }

    fn publish_component_add_page(&mut self) -> Result<(), String> {
        let state = &self.inspector_component_add;
        let (options, disabled, previous, next, page, has_options) = match &state.catalog {
            Some(catalog) => (
                catalog.entries.iter().map(|entry| UiValue::String(entry.type_path.clone())).collect(),
                catalog.entries.iter().filter(|entry| entry.already_present)
                    .map(|entry| UiValue::String(entry.type_path.clone())).collect(),
                catalog.page > 0, (catalog.page + 1) * PAGE_SIZE < catalog.total,
                format!("{}/{}", catalog.page + 1, catalog.total.div_ceil(PAGE_SIZE).max(1)),
                !catalog.entries.is_empty(),
            ),
            None => (Vec::new(), Vec::new(), false, false, "0/0".into(), false),
        };
        let selected = state.selected.as_ref();
        let path = selected.map(|selected| selected.spec.type_path.clone()).unwrap_or_default();
        let name = selected.map(|selected| selected.display_name.clone()).unwrap_or_default();
        let required = selected.map(|selected| selected.required_fields.join(", ")).unwrap_or_default();
        let available = selected.is_some();
        for (control, property, value) in [
            (TYPE, "options", UiValue::Array(options)),
            (TYPE, "disabled_options", UiValue::Array(disabled)),
            (TYPE, "value", UiValue::String(path.clone())),
            (TYPE, "value_text", UiValue::String(path)),
            (TYPE, "disabled", UiValue::Bool(!has_options)),
            ("WorkbenchComponentAddName", "text", UiValue::String(name)),
            ("WorkbenchComponentAddRequired", "text", UiValue::String(required)),
            ("WorkbenchComponentAddPage", "text", UiValue::String(page)),
            ("WorkbenchComponentAddPrevious", "disabled", UiValue::Bool(!previous)),
            ("WorkbenchComponentAddNext", "disabled", UiValue::Bool(!next)),
            ("WorkbenchComponentAddConfirm", "disabled", UiValue::Bool(!available)),
            (VALUES, "disabled", UiValue::Bool(!available)),
        ] {
            self.mutate_control_property(control, property, value).map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    fn page_component_add(&mut self, runtime: &EditorHostEventController, next: bool) -> Result<(), String> {
        let Some(catalog) = &self.inspector_component_add.catalog else { return Ok(()); };
        let page = if next {
            if (catalog.page + 1) * PAGE_SIZE >= catalog.total { return Ok(()); }
            catalog.page + 1
        } else {
            let Some(page) = catalog.page.checked_sub(1) else { return Ok(()); };
            page
        };
        match runtime.inspector_component_add_catalog(page, Some(&catalog.scope)) {
            Ok(catalog) => {
                self.inspector_component_add.catalog = Some(catalog);
                self.seed_first_component_add(runtime)?;
                self.close_popup(TYPE).map_err(|error| error.to_string())?;
                self.publish_component_add_page()?;
            }
            Err(_) => self.component_add_feedback("editor.inspector.component_add.rejected")?,
        }
        self.refresh_component_add()
    }

    pub(crate) fn select_inspector_component_add_type(
        &mut self, runtime: &EditorHostEventController, control: &str, option: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if control != TYPE { return None; }
        Some((|| {
            let Some(catalog) = &self.inspector_component_add.catalog else { return Ok(component_add_paint()); };
            if !catalog.entries.iter().any(|entry| !entry.already_present && entry.type_path == option) {
                return Ok(component_add_paint());
            }
            let selected = match runtime.inspector_component_add_template(&catalog.scope, option) {
                Ok(selected) => selected,
                Err(_) => {
                    self.component_add_feedback("editor.inspector.component_add.rejected")?;
                    self.refresh_component_add()?;
                    return Ok(component_add_paint());
                }
            };
            if !self.select_dropdown_option(TYPE, option).map_err(|error| error.to_string())? {
                return Ok(component_add_paint());
            }
            self.inspector_component_add.selected = Some(selected);
            self.seed_component_add_draft()?;
            self.publish_component_add_page()?;
            self.component_add_feedback("editor.inspector.component_add.initial_values")?;
            self.refresh_component_add()?;
            Ok(component_add_paint())
        })())
    }

    pub(crate) fn edit_inspector_component_add(
        &mut self, control: &str, binding: &str, value: &str,
    ) -> Result<Option<bool>, BuiltinHostWindowTemplateBridgeError> {
        if !matches!(binding, "Inspector/ComponentAddValuesEdit" | "Inspector/ComponentAddValuesCommit") {
            return Ok(None);
        }
        if control != VALUES || self.inspector_component_add.selected.is_none() { return Ok(Some(false)); }
        self.inspector_component_add.draft = value.to_owned();
        self.mutate_control_property(VALUES, "query", UiValue::String(value.to_owned()))?;
        self.template_surface.refresh_after_state_change(self.runtime.as_ref())?;
        Ok(Some(true))
    }

    fn confirm_component_add(&mut self, runtime: &EditorHostEventController) -> Result<UiHostEventEffects, String> {
        let state = &self.inspector_component_add;
        let Some(catalog) = &state.catalog else { return Ok(component_add_paint()); };
        let Some(selected) = &state.selected else { return Ok(component_add_paint()); };
        let value = match serde_json::from_str(&state.draft) {
            Ok(value) => value,
            Err(_) => {
                self.component_add_feedback("editor.inspector.component_add.invalid_json")?;
                self.refresh_component_add()?;
                return Ok(component_add_paint());
            }
        };
        let mut component = selected.spec.clone();
        component.value = value;
        let request = AddComponentRequest { scope: catalog.scope.clone(), component };
        let invocation = EditorOperationInvocation::parse(ADD_COMPONENT_OPERATION)
            .map_err(|error| error.to_string())?
            .with_arguments(serde_json::to_value(&request).map_err(|error| error.to_string())?);
        let mut effects = component_add_paint();
        match runtime.invoke_operation(EditorOperationSource::UiBinding, invocation) {
            Ok(record) => {
                apply_record_effects(&mut effects, &record);
                self.close_component_add()?;
            }
            Err(_) => {
                // Preserve the draft; typed diagnostics stay in the actual operation
                // record. Visible feedback comes from the locale catalog.
                self.component_add_feedback("editor.inspector.component_add.rejected")?;
                self.refresh_component_add()?;
            }
        }
        Ok(effects)
    }

    fn close_component_add(&mut self) -> Result<(), String> {
        let previous = self.inspector_component_add.previous_focus;
        self.inspector_component_add = InspectorComponentAddState::default();
        self.close_popup(TYPE).map_err(|error| error.to_string())?;
        self.publish_component_add_page()?;
        self.mutate_control_property(VALUES, "query", UiValue::String(String::new())).map_err(|error| error.to_string())?;
        self.set_visible(PANEL, false).map_err(|error| error.to_string())?;
        self.refresh_component_add()?;
        if let Some(previous) = previous.filter(|node| self.surface().tree.nodes.contains_key(node)) {
            let _ = self.template_surface.surface.focus_node(previous);
        }
        Ok(())
    }

    fn component_add_feedback(&mut self, key: &str) -> Result<(), String> {
        let text = self.i18n.translate(key).to_string();
        self.mutate_control_property("WorkbenchComponentAddFeedback", "text", UiValue::String(text))
            .map_err(|error| error.to_string())
    }
    fn refresh_component_add(&mut self) -> Result<(), String> {
        self.template_surface.refresh_after_state_change(self.runtime.as_ref()).map_err(|error| error.to_string())
    }
}

fn component_add_paint() -> UiHostEventEffects {
    let mut effects = UiHostEventEffects::default();
    effects.request_paint_only();
    effects
}
