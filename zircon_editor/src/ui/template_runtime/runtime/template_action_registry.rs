use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::{dispatch::UiTemplateActionInvocation, template::UiActionRef};

use super::plugin_documents::EditorPluginV2DocumentOwner;
use super::template_action_slot::TemplateActionSlot;

#[derive(Default)]
pub(super) struct TemplateActionRegistry {
    actions: BTreeMap<String, TemplateActionSlot>,
    pane_binding_epochs: BTreeMap<String, u128>,
    // One mutable control-state snapshot serves every action binding in a pane/document generation.
    control_attributes_by_pane:
        BTreeMap<TemplateActionPaneKey, BTreeMap<String, BTreeMap<String, Value>>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct TemplateActionPaneKey {
    pane_id: String,
    document_id: String,
    plugin_owner: Option<EditorPluginV2DocumentOwner>,
}

impl TemplateActionPaneKey {
    pub(super) fn new(
        pane_id: &str,
        document_id: &str,
        plugin_owner: Option<&EditorPluginV2DocumentOwner>,
    ) -> Self {
        Self {
            pane_id: pane_id.to_string(),
            document_id: document_id.to_string(),
            plugin_owner: plugin_owner.cloned(),
        }
    }

    fn from_owned(
        pane_id: &str,
        document_id: &str,
        plugin_owner: Option<EditorPluginV2DocumentOwner>,
    ) -> Self {
        Self {
            pane_id: pane_id.to_string(),
            document_id: document_id.to_string(),
            plugin_owner,
        }
    }

    pub(super) fn pane_id(&self) -> &str {
        &self.pane_id
    }

    pub(super) fn document_id(&self) -> &str {
        &self.document_id
    }

    pub(super) fn plugin_owner(&self) -> Option<&EditorPluginV2DocumentOwner> {
        self.plugin_owner.as_ref()
    }
}

impl TemplateActionRegistry {
    pub(super) fn remove_pane(&mut self, pane_id: &str) {
        self.advance_pane_binding_epoch(pane_id);
        let prefix = format!("template-v2/{pane_id}/");
        self.actions.retain(|token, _| !token.starts_with(&prefix));
        self.control_attributes_by_pane
            .retain(|key, _| key.pane_id != pane_id);
    }

    fn advance_pane_binding_epoch(&mut self, pane_id: &str) {
        let pane_epoch = self
            .pane_binding_epochs
            .entry(pane_id.to_string())
            .or_default();
        *pane_epoch = pane_epoch
            .checked_add(1)
            .expect("template action pane epoch must not wrap");
    }

    pub(super) fn remove_document(&mut self, document_id: &str) {
        self.actions
            .retain(|_, slot| slot.document_id() != document_id);
        self.control_attributes_by_pane
            .retain(|key, _| key.document_id != document_id);
    }

    pub(super) fn bind(
        &mut self,
        pane_id: &str,
        document_id: &str,
        binding_id: &str,
        plugin_owner: Option<EditorPluginV2DocumentOwner>,
        source_attributes: BTreeMap<String, Value>,
        action_source: UiActionRef,
        control_attributes: BTreeMap<String, BTreeMap<String, Value>>,
    ) -> String {
        self.bind_for_control(
            pane_id,
            document_id,
            binding_id,
            plugin_owner,
            None,
            source_attributes,
            action_source,
            control_attributes,
        )
    }

    pub(super) fn bind_for_control(
        &mut self,
        pane_id: &str,
        document_id: &str,
        binding_id: &str,
        plugin_owner: Option<EditorPluginV2DocumentOwner>,
        control_id: Option<&str>,
        source_attributes: BTreeMap<String, Value>,
        action_source: UiActionRef,
        control_attributes: BTreeMap<String, BTreeMap<String, Value>>,
    ) -> String {
        let pane_epoch = self
            .pane_binding_epochs
            .get(pane_id)
            .copied()
            .unwrap_or_default();
        let token = template_action_token(
            pane_id,
            document_id,
            plugin_owner.as_ref(),
            pane_epoch,
            binding_id,
        );
        let pane_key = TemplateActionPaneKey::from_owned(pane_id, document_id, plugin_owner);
        self.control_attributes_by_pane
            .entry(pane_key.clone())
            .or_insert(control_attributes);
        self.actions.insert(
            token.clone(),
            TemplateActionSlot::new(pane_key, control_id, source_attributes, action_source),
        );
        token
    }

    pub(super) fn rebind_pane(
        &mut self,
        pane_id: &str,
        document_id: &str,
        plugin_owner: Option<&EditorPluginV2DocumentOwner>,
        mut control_attributes: BTreeMap<String, BTreeMap<String, Value>>,
    ) -> BTreeMap<String, BTreeMap<String, Value>> {
        let key = TemplateActionPaneKey::new(pane_id, document_id, plugin_owner);
        let previous_control_attributes = self.control_attributes_by_pane.remove(&key);
        self.remove_pane(pane_id);
        if let Some(previous_control_attributes) = previous_control_attributes {
            restore_current_table_selection(&mut control_attributes, previous_control_attributes);
        }
        self.control_attributes_by_pane
            .insert(key, control_attributes.clone());
        control_attributes
    }

    pub(super) fn update_control_attributes_for_pane(
        &mut self,
        pane_id: &str,
        control_id: &str,
        attributes: &BTreeMap<String, Value>,
    ) -> bool {
        let mut matching_keys = self
            .control_attributes_by_pane
            .keys()
            .filter(|key| key.pane_id == pane_id)
            .cloned();
        let Some(key) = matching_keys.next() else {
            return false;
        };
        if matching_keys.next().is_some() {
            return false;
        }
        let control_attributes = self
            .control_attributes_by_pane
            .get_mut(&key)
            .expect("matching template action pane key should remain registered");
        let control_attributes = control_attributes
            .entry(control_id.to_string())
            .or_default();
        control_attributes.extend(attributes.clone());
        if attributes.contains_key("rows") || attributes.contains_key("row_identity_field") {
            clear_stale_table_selection(control_attributes);
        }
        for slot in self
            .actions
            .values_mut()
            .filter(|slot| slot.pane_id() == pane_id && slot.control_id() == Some(control_id))
        {
            slot.update_source_attributes(attributes);
        }
        true
    }

    pub(super) fn select_table_row(
        &mut self,
        pane_id: &str,
        control_id: &str,
        source_index: i32,
        identity_kind: &str,
        identity_text: &str,
    ) -> bool {
        let Some(control_attributes) = self.control_attributes_for_pane_mut(pane_id, control_id)
        else {
            return false;
        };
        let Some(identity) = table_row_identity(control_attributes, source_index) else {
            return false;
        };
        // Only the identity from this pane's current projection may mutate selection state.
        if scalar_identity(&identity) != Some((identity_kind, identity_text.to_string())) {
            return false;
        }
        control_attributes.insert(
            "selected_index".to_string(),
            Value::Integer(i64::from(source_index)),
        );
        control_attributes.insert("selected_row_identity".to_string(), identity);
        true
    }

    fn control_attributes_for_pane_mut(
        &mut self,
        pane_id: &str,
        control_id: &str,
    ) -> Option<&mut BTreeMap<String, Value>> {
        let mut matching_keys = self
            .control_attributes_by_pane
            .keys()
            .filter(|key| key.pane_id == pane_id)
            .cloned();
        let key = matching_keys.next()?;
        if matching_keys.next().is_some() {
            return None;
        }
        self.control_attributes_by_pane
            .get_mut(&key)
            .map(|control_attributes| {
                control_attributes
                    .entry(control_id.to_string())
                    .or_default()
            })
    }

    pub(super) fn contains_token(&self, token: &str) -> bool {
        self.actions.contains_key(token)
    }

    pub(super) fn action_for_token<F>(
        &self,
        token: &str,
        plugin_owner_for_document: F,
    ) -> Option<UiTemplateActionInvocation>
    where
        F: FnOnce(&str) -> Option<EditorPluginV2DocumentOwner>,
    {
        let slot = self.actions.get(token)?;
        if action_source_is_disabled(slot.source_attributes()) {
            return None;
        }
        let control_attributes = self.control_attributes_by_pane.get(slot.pane_key())?;
        let compiled_action = slot.compiled_action()?;
        if let Some(expected_owner) = slot.plugin_owner() {
            (plugin_owner_for_document(slot.document_id()).as_ref() == Some(expected_owner))
                .then(|| compiled_action.resolve(slot.source_attributes(), control_attributes))
                .flatten()
        } else {
            compiled_action.resolve(slot.source_attributes(), control_attributes)
        }
    }
}

#[cfg(test)]
#[path = "template_action_registry/tests/optimization_batch_js_editor658_tests.rs"]
mod optimization_batch_js_editor658_tests;

fn action_source_is_disabled(attributes: &BTreeMap<String, Value>) -> bool {
    attributes.get("disabled") == Some(&Value::Boolean(true))
        || attributes.get("enabled") == Some(&Value::Boolean(false))
}

fn restore_current_table_selection(
    control_attributes: &mut BTreeMap<String, BTreeMap<String, Value>>,
    previous_control_attributes: BTreeMap<String, BTreeMap<String, Value>>,
) {
    for (control_id, previous_attributes) in previous_control_attributes {
        let Some(source_index) = previous_attributes
            .get("selected_index")
            .and_then(Value::as_integer)
            .and_then(|value| i32::try_from(value).ok())
        else {
            continue;
        };
        let Some(previous_identity) = previous_attributes.get("selected_row_identity") else {
            continue;
        };
        let Some(current_attributes) = control_attributes.get_mut(&control_id) else {
            continue;
        };
        let Some(current_identity) = table_row_identity(current_attributes, source_index) else {
            continue;
        };
        if &current_identity != previous_identity {
            continue;
        }
        current_attributes.insert(
            "selected_index".to_string(),
            Value::Integer(i64::from(source_index)),
        );
        current_attributes.insert("selected_row_identity".to_string(), current_identity);
    }
}

fn clear_stale_table_selection(control_attributes: &mut BTreeMap<String, Value>) {
    let selected_index = control_attributes
        .get("selected_index")
        .and_then(Value::as_integer)
        .and_then(|value| i32::try_from(value).ok());
    let selected_identity = control_attributes.get("selected_row_identity");
    let selection_is_current = selected_index
        .and_then(|source_index| table_row_identity(control_attributes, source_index))
        .as_ref()
        .is_some_and(|identity| Some(identity) == selected_identity);
    if !selection_is_current {
        control_attributes.remove("selected_index");
        control_attributes.remove("selected_row_identity");
    }
}

fn table_row_identity(
    control_attributes: &BTreeMap<String, Value>,
    source_index: i32,
) -> Option<Value> {
    let identity_field = control_attributes
        .get("row_identity_field")
        .and_then(Value::as_str)?;
    control_attributes
        .get("rows")
        .and_then(Value::as_array)
        .and_then(|rows| {
            usize::try_from(source_index)
                .ok()
                .and_then(|index| rows.get(index))
        })
        .and_then(Value::as_table)
        .and_then(|row| row.get(identity_field))
        .cloned()
}

fn scalar_identity(value: &Value) -> Option<(&'static str, String)> {
    match value {
        Value::String(value) => Some(("string", value.clone())),
        Value::Integer(value) => Some(("integer", value.to_string())),
        Value::Float(value) => Some(("float", value.to_string())),
        Value::Boolean(value) => Some(("boolean", value.to_string())),
        _ => None,
    }
}

fn template_action_token(
    pane_id: &str,
    document_id: &str,
    plugin_owner: Option<&EditorPluginV2DocumentOwner>,
    pane_epoch: u128,
    binding_id: &str,
) -> String {
    let owner_generation = plugin_owner
        .map(EditorPluginV2DocumentOwner::generation)
        .unwrap_or_default();
    format!("template-v2/{pane_id}/{document_id}/g{owner_generation}/e{pane_epoch}/{binding_id}")
}

#[cfg(test)]
#[path = "tests/template_action_registry.rs"]
mod tests;
