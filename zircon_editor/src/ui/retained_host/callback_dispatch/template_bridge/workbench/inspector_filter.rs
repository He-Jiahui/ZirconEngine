use std::sync::Arc;

use zircon_runtime_interface::ui::component::UiValue;

use crate::ui::workbench::snapshot::{
    InspectorPluginComponentPropertySnapshot, InspectorPluginComponentSnapshot,
};

use super::{
    component_property_rows::component_property_item_keys,
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const FILTER_CONTROL: &str = "WorkbenchInspectorFilter";
const FILTER_EDIT: &str = "Workbench/InspectorSearchEdit";
const FILTER_COMMIT: &str = "Workbench/InspectorSearchCommit";
const FILTER_EDIT_ACTION: &str = "workbench.inspector.search.edit";
const FILTER_COMMIT_ACTION: &str = "workbench.inspector.search.commit";
const INSPECTOR_METADATA: &str = "WorkbenchInspectorMetadata";
const INSPECTOR_TRANSFORM: &str = "WorkbenchInspectorTransform";
const INSPECTOR_COMPONENT: &str = "WorkbenchInspectorMesh";
const ADD_COMPONENT: &str = "WorkbenchAddComponent";
const FILTER_EMPTY: &str = "WorkbenchInspectorFilterEmpty";

const METADATA_TERMS: &[&str] = &["metadata", "render", "layer", "mask"];
const TRANSFORM_TERMS: &[&str] = &["transform", "position", "rotation", "scale", "translation"];

pub(super) fn is_inspector_filter_action(action_id: &str) -> bool {
    matches!(action_id, FILTER_EDIT_ACTION | FILTER_COMMIT_ACTION)
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn edit_inspector_filter(
        &mut self,
        control_id: &str,
        binding_id: &str,
        value: &str,
    ) -> Result<Option<bool>, BuiltinHostWindowTemplateBridgeError> {
        if !matches!(binding_id, FILTER_EDIT | FILTER_COMMIT) {
            return Ok(None);
        }
        if !control_id.is_empty() && control_id != FILTER_CONTROL {
            return Ok(Some(false));
        }

        self.mutate_control_property(FILTER_CONTROL, "query", UiValue::String(value.to_string()))?;
        self.apply_inspector_filter()?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(Some(true))
    }

    pub(super) fn apply_inspector_filter_action(
        &mut self,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if is_inspector_filter_action(action_id) {
            self.apply_inspector_filter()?;
        }
        Ok(())
    }

    pub(super) fn set_inspector_filter_source(
        &mut self,
        has_selection: bool,
        component: Option<&InspectorPluginComponentSnapshot>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.inspector_has_selection = has_selection;
        self.inspector_has_component = component.is_some();
        self.inspector_component_label = component
            .map(|component| non_empty_label(&component.display_name, "Component"))
            .unwrap_or_default();
        self.inspector_source_properties = component
            .map(|component| Arc::from(component.properties.clone()))
            .unwrap_or_else(|| Arc::from([]));
        self.component_customization_available = component
            .map(|component| component.customization_available)
            .unwrap_or(false);
        self.apply_inspector_filter()
    }

    fn apply_inspector_filter(&mut self) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let query = self
            .control_string(FILTER_CONTROL, "query")
            .unwrap_or_default();
        let query = query.trim();
        let has_selection = self.inspector_has_selection;
        let metadata_visible = has_selection && term_group_matches(METADATA_TERMS, query);
        let transform_visible = has_selection && term_group_matches(TRANSFORM_TERMS, query);
        let component_category_matches = query.is_empty()
            || contains_ascii_case_insensitive(&self.inspector_component_label, query);
        let source_properties = self.inspector_source_properties.clone();
        let visible_properties = if has_selection && self.inspector_has_component {
            source_properties
                .iter()
                .filter(|property| {
                    component_category_matches || component_property_matches(property, query)
                })
                .cloned()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let component_visible = has_selection
            && self.inspector_has_component
            && (component_category_matches || !visible_properties.is_empty());
        let add_component_visible = has_selection && query.is_empty();
        let empty_visible = has_selection
            && !query.is_empty()
            && !metadata_visible
            && !transform_visible
            && !component_visible;

        self.mutate_control_property(FILTER_CONTROL, "disabled", UiValue::Bool(!has_selection))?;
        self.set_visible(INSPECTOR_METADATA, metadata_visible)?;
        self.set_visible(INSPECTOR_TRANSFORM, transform_visible)?;
        self.set_visible(INSPECTOR_COMPONENT, component_visible)?;
        self.set_visible(ADD_COMPONENT, add_component_visible)?;
        self.set_visible(FILTER_EMPTY, empty_visible)?;

        let visible_properties: Arc<[InspectorPluginComponentPropertySnapshot]> =
            if component_visible {
                Arc::from(visible_properties)
            } else {
                Arc::from([])
            };
        let item_keys = component_property_item_keys(visible_properties.as_ref());
        let bindings = self.component_property_row_bindings(item_keys.as_slice())?;
        self.component_properties = visible_properties;
        self.component_property_keys = Arc::from(item_keys);
        let properties = self.component_properties.clone();
        for binding in &bindings {
            self.sync_component_property_binding(
                binding,
                properties.as_ref(),
                self.component_customization_available,
            )?;
        }
        Ok(())
    }
}

fn term_group_matches(terms: &[&str], query: &str) -> bool {
    query.is_empty()
        || terms
            .iter()
            .any(|term| contains_ascii_case_insensitive(term, query))
}

fn component_property_matches(
    property: &InspectorPluginComponentPropertySnapshot,
    query: &str,
) -> bool {
    query.is_empty()
        || [
            property.field_id.as_str(),
            property.name.as_str(),
            property.label.as_str(),
            property.value_kind.as_str(),
        ]
        .iter()
        .any(|candidate| contains_ascii_case_insensitive(candidate, query))
}

fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    needle.is_empty()
        || haystack
            .as_bytes()
            .windows(needle.len())
            .any(|candidate| candidate.eq_ignore_ascii_case(needle.as_bytes()))
}

fn non_empty_label(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_matching_is_ascii_case_insensitive_and_empty_queries_match() {
        assert!(contains_ascii_case_insensitive("Mesh Renderer", "renderer"));
        assert!(contains_ascii_case_insensitive("Render Layer", "LAYER"));
        assert!(contains_ascii_case_insensitive("Transform", ""));
        assert!(!contains_ascii_case_insensitive("Position", "material"));
    }
}
