use std::collections::BTreeMap;

use crate::ui::binding::EditorUiBinding;
use toml::Value;
use zircon_runtime_interface::ui::{event_ui::UiRouteId, template::UiActionRef};

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
                metadata.attributes.extend(node.attributes.clone());
                metadata.style_tokens.extend(node.style_tokens.clone());
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
        attributes.extend(metadata.attributes.clone());
        style_tokens.extend(metadata.style_tokens.clone());
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
mod tests {
    use super::*;

    #[test]
    fn surface_metadata_index_preserves_preorder_last_write_wins() {
        let projection = RetainedUiProjection {
            document_id: "test.index".to_string(),
            bindings: Vec::new(),
            root: RetainedUiNodeProjection {
                component: "Root".to_string(),
                control_id: None,
                attributes: BTreeMap::new(),
                style_tokens: BTreeMap::new(),
                binding_ids: Vec::new(),
                children: vec![
                    RetainedUiNodeProjection {
                        component: "Button".to_string(),
                        control_id: Some("SharedControl".to_string()),
                        attributes: BTreeMap::from([
                            ("preserved".to_string(), Value::Boolean(true)),
                            ("winner".to_string(), Value::String("first".to_string())),
                        ]),
                        style_tokens: BTreeMap::from([("accent".to_string(), "first".to_string())]),
                        binding_ids: Vec::new(),
                        children: Vec::new(),
                    },
                    RetainedUiNodeProjection {
                        component: "Button".to_string(),
                        control_id: Some("SharedControl".to_string()),
                        attributes: BTreeMap::from([(
                            "winner".to_string(),
                            Value::String("second".to_string()),
                        )]),
                        style_tokens: BTreeMap::from([(
                            "accent".to_string(),
                            "second".to_string(),
                        )]),
                        binding_ids: Vec::new(),
                        children: Vec::new(),
                    },
                ],
            },
        };

        let index = projection.surface_metadata_index();
        let (attributes, style_tokens) = index.metadata_for("SharedControl").unwrap();
        assert_eq!(attributes.get("preserved"), Some(&Value::Boolean(true)));
        assert_eq!(
            attributes.get("winner"),
            Some(&Value::String("second".to_string()))
        );
        assert_eq!(style_tokens.get("accent"), Some(&"second".to_string()));
    }
}
