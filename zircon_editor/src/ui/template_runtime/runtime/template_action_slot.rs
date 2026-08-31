use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiActionRef;

use super::compiled_template_action::CompiledTemplateAction;
use super::plugin_documents::EditorPluginV2DocumentOwner;

#[derive(Clone, Debug)]
pub(super) struct TemplateActionSlot {
    document_id: String,
    pane_id: String,
    control_id: Option<String>,
    plugin_owner: Option<EditorPluginV2DocumentOwner>,
    source_attributes: BTreeMap<String, Value>,
    compiled_action: Option<CompiledTemplateAction>,
}

impl TemplateActionSlot {
    pub(super) fn new(
        pane_id: impl Into<String>,
        document_id: impl Into<String>,
        control_id: Option<&str>,
        plugin_owner: Option<EditorPluginV2DocumentOwner>,
        source_attributes: BTreeMap<String, Value>,
        action_source: UiActionRef,
    ) -> Self {
        Self {
            document_id: document_id.into(),
            pane_id: pane_id.into(),
            control_id: control_id.map(str::to_string),
            plugin_owner,
            source_attributes,
            compiled_action: CompiledTemplateAction::compile(&action_source),
        }
    }

    pub(super) fn document_id(&self) -> &str {
        &self.document_id
    }

    pub(super) fn pane_id(&self) -> &str {
        &self.pane_id
    }

    pub(super) fn control_id(&self) -> Option<&str> {
        self.control_id.as_deref()
    }

    pub(super) fn plugin_owner(&self) -> Option<&EditorPluginV2DocumentOwner> {
        self.plugin_owner.as_ref()
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
