use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiActionRef;

use super::compiled_template_action::CompiledTemplateAction;
use super::plugin_documents::EditorPluginV2DocumentOwner;
use super::template_action_registry::TemplateActionPaneKey;

#[derive(Clone, Debug)]
pub(super) struct TemplateActionSlot {
    pane_key: TemplateActionPaneKey,
    control_id: Option<String>,
    source_attributes: BTreeMap<String, Value>,
    compiled_action: Option<CompiledTemplateAction>,
}

impl TemplateActionSlot {
    pub(super) fn new(
        pane_key: TemplateActionPaneKey,
        control_id: Option<&str>,
        source_attributes: BTreeMap<String, Value>,
        action_source: UiActionRef,
    ) -> Self {
        Self {
            pane_key,
            control_id: control_id.map(str::to_string),
            source_attributes,
            compiled_action: CompiledTemplateAction::compile(&action_source),
        }
    }

    pub(super) fn document_id(&self) -> &str {
        self.pane_key.document_id()
    }

    pub(super) fn pane_id(&self) -> &str {
        self.pane_key.pane_id()
    }

    pub(super) fn control_id(&self) -> Option<&str> {
        self.control_id.as_deref()
    }

    pub(super) fn plugin_owner(&self) -> Option<&EditorPluginV2DocumentOwner> {
        self.pane_key.plugin_owner()
    }

    pub(super) fn pane_key(&self) -> &TemplateActionPaneKey {
        &self.pane_key
    }

    pub(super) fn source_attributes(&self) -> &BTreeMap<String, Value> {
        &self.source_attributes
    }

    pub(super) fn update_source_attributes(&mut self, attributes: &BTreeMap<String, Value>) {
        self.source_attributes.extend(attributes.clone());
    }

    pub(super) fn compiled_action(&self) -> Option<&CompiledTemplateAction> {
        self.compiled_action.as_ref()
    }
}
