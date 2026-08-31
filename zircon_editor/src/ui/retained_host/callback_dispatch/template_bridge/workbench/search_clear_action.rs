use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiPointerActivationPhase, UiPointerButton, UiPointerRoute},
    tree::UiTemplateNodeMetadata,
};

use crate::ui::retained_host::host_contract::search_field_clear_action_hit_test;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const QUERY: &str = "query";
const SEARCH_INPUT_CLASS: &str = "workbench-search-input";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn clear_search_field_from_pointer_route(
        &mut self,
        route: &UiPointerRoute,
    ) -> Result<Option<String>, BuiltinHostWindowTemplateBridgeError> {
        if route.activation_phase != UiPointerActivationPhase::PrimaryRelease
            || route.button != Some(UiPointerButton::Primary)
        {
            return Ok(None);
        }

        let Some(node_id) = route.click_target else {
            return Ok(None);
        };
        let Some(target) = self.search_clear_action_target(node_id) else {
            return Ok(None);
        };
        let query = self
            .control_string(&target.control_id, QUERY)
            .unwrap_or_default();
        if query.trim().is_empty() || !search_field_clear_action_hit_test(target.frame, route.point)
        {
            return Ok(None);
        }

        self.mutate_control_property(&target.control_id, QUERY, UiValue::String(String::new()))?;
        Ok(Some(target.control_id))
    }

    fn search_clear_action_target(&self, node_id: UiNodeId) -> Option<SearchClearActionTarget> {
        let node = self.template_surface.surface.tree.nodes.get(&node_id)?;
        let metadata = node.template_metadata.as_ref()?;
        let control_id = metadata.control_id.as_deref()?;
        if !metadata_is_clearable_search_input(metadata) {
            return None;
        }
        let frame = self
            .template_surface
            .surface
            .arranged_tree
            .get(node_id)
            .map(|node| node.frame)?;
        Some(SearchClearActionTarget {
            control_id: control_id.to_string(),
            frame,
        })
    }
}

struct SearchClearActionTarget {
    control_id: String,
    frame: UiFrame,
}

fn metadata_is_clearable_search_input(metadata: &UiTemplateNodeMetadata) -> bool {
    let is_search_input = metadata.component == "SearchField"
        || metadata
            .classes
            .iter()
            .any(|class| class.as_str() == SEARCH_INPUT_CLASS);
    is_search_input
        && metadata
            .attributes
            .get("has_clear_action")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false)
        && metadata
            .attributes
            .get("enabled")
            .and_then(toml::Value::as_bool)
            .unwrap_or(true)
        && !metadata
            .attributes
            .get("disabled")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false)
}
