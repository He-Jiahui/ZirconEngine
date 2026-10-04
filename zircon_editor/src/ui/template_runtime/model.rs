use std::collections::BTreeMap;

use crate::ui::binding::EditorUiBinding;
use toml::Value;
use zircon_runtime_interface::ui::{
    event_ui::UiRouteId, template::UiActionRef, v2::UiTemplateNodeInstancePathStep,
};

#[derive(Clone, Debug, PartialEq)]
pub struct RetainedUiBindingProjection {
    pub binding_id: String,
    pub binding: EditorUiBinding,
    pub route_id: Option<UiRouteId>,
    pub template_action: Option<UiActionRef>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RetainedUiNodeProjection {
    pub component: String,
    pub control_id: Option<String>,
    pub source_path: Option<String>,
    pub source_node_id: Option<String>,
    pub instance_path: Option<Vec<UiTemplateNodeInstancePathStep>>,
    pub attributes: BTreeMap<String, Value>,
    pub style_tokens: BTreeMap<String, String>,
    pub binding_ids: Vec<String>,
    pub children: Vec<RetainedUiNodeProjection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RetainedUiProjection {
    pub document_id: String,
    pub root: RetainedUiNodeProjection,
    pub bindings: Vec<RetainedUiBindingProjection>,
}

/// Immutable authored metadata used when a retained surface patches an existing node.
///
/// The workbench keeps its source projection for its entire lifetime. Building this
/// index once keeps a one-node semantic patch from walking the complete authored tree.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RetainedUiProjectionSurfaceMetadataIndex {
    by_control_id: BTreeMap<String, RetainedUiProjectionSurfaceMetadata>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RetainedUiProjectionSurfaceMetadata {
    attributes: BTreeMap<String, Value>,
    style_tokens: BTreeMap<String, String>,
}

impl RetainedUiProjection {
    pub(crate) fn surface_metadata_index(&self) -> RetainedUiProjectionSurfaceMetadataIndex {
        let mut by_control_id = BTreeMap::new();
        let mut stack = vec![&self.root];
        while let Some(node) = stack.pop() {
            if let Some(control_id) = node.control_id.as_ref() {
                let metadata = by_control_id
                    .entry(control_id.clone())
                    .or_insert_with(RetainedUiProjectionSurfaceMetadata::default);
                metadata.attributes.extend(
                    node.attributes
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone())),
                );
                metadata.style_tokens.extend(
                    node.style_tokens
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone())),
                );
            }
            stack.extend(node.children.iter().rev());
        }
        RetainedUiProjectionSurfaceMetadataIndex { by_control_id }
    }
}

impl RetainedUiProjectionSurfaceMetadataIndex {
    pub(crate) fn apply_to(
        &self,
        control_id: Option<&str>,
        attributes: &mut BTreeMap<String, Value>,
        style_tokens: &mut BTreeMap<String, String>,
    ) {
        let Some(metadata) = control_id.and_then(|control_id| self.by_control_id.get(control_id))
        else {
            return;
        };
        // Authored metadata supplements the live surface; it must not reset runtime values.
        for (key, value) in &metadata.attributes {
            attributes
                .entry(key.clone())
                .or_insert_with(|| value.clone());
        }
        style_tokens.extend(
            metadata
                .style_tokens
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
    }

    #[cfg(test)]
    pub(crate) fn metadata_for(
        &self,
        control_id: &str,
    ) -> Option<(&BTreeMap<String, Value>, &BTreeMap<String, String>)> {
        let metadata = self.by_control_id.get(control_id)?;
        Some((&metadata.attributes, &metadata.style_tokens))
    }
}

#[cfg(test)]
#[path = "tests/model.rs"]
mod tests;

#[cfg(test)]
#[path = "model/tests/optimization_tests.rs"]
mod optimization_tests;
