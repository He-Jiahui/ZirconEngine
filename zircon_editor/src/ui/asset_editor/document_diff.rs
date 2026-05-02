use zircon_runtime_interface::ui::template::UiAssetDocument;

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct UiAssetDocumentDiff {
    target: Option<UiAssetDocument>,
}

impl UiAssetDocumentDiff {
    pub fn between(current: &UiAssetDocument, target: &UiAssetDocument) -> Self {
        Self {
            target: (current != target).then(|| target.clone()),
        }
    }

    pub fn apply_to(&self, document: &mut UiAssetDocument) -> bool {
        let Some(target) = &self.target else {
            return false;
        };
        if *document == *target {
            return false;
        }
        *document = target.clone();
        true
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use toml::Value;
    use zircon_runtime_interface::ui::template::{
        UiAssetDocument, UiAssetHeader, UiAssetImports, UiAssetKind, UiChildMount,
        UiNodeDefinition, UiNodeDefinitionKind,
    };

    use super::UiAssetDocumentDiff;

    #[test]
    fn document_diff_replaces_recursive_document_when_changed() {
        let before = fixture_document("Ready");
        let after = fixture_document("Saved");

        let diff = UiAssetDocumentDiff::between(&before, &after);
        let mut patched = before.clone();

        assert!(diff.apply_to(&mut patched));
        assert_eq!(patched, after);
    }

    #[test]
    fn document_diff_noops_when_documents_match() {
        let document = fixture_document("Ready");
        let diff = UiAssetDocumentDiff::between(&document, &document);
        let mut patched = document.clone();

        assert!(!diff.apply_to(&mut patched));
        assert_eq!(patched, document);
    }

    fn fixture_document(text: &str) -> UiAssetDocument {
        UiAssetDocument {
            asset: UiAssetHeader {
                kind: UiAssetKind::Layout,
                id: "editor.test.layout".to_string(),
                version: 1,
                display_name: "Test Layout".to_string(),
            },
            imports: UiAssetImports::default(),
            tokens: BTreeMap::new(),
            root: Some(UiNodeDefinition {
                node_id: "root".to_string(),
                kind: UiNodeDefinitionKind::Native,
                widget_type: Some("VerticalBox".to_string()),
                component: None,
                component_ref: None,
                component_api_version: None,
                slot_name: None,
                control_id: Some("Root".to_string()),
                classes: Vec::new(),
                params: BTreeMap::new(),
                props: BTreeMap::new(),
                layout: None,
                bindings: Vec::new(),
                style_overrides: Default::default(),
                children: vec![UiChildMount {
                    mount: None,
                    slot: BTreeMap::new(),
                    node: UiNodeDefinition {
                        node_id: "status".to_string(),
                        kind: UiNodeDefinitionKind::Native,
                        widget_type: Some("Label".to_string()),
                        component: None,
                        component_ref: None,
                        component_api_version: None,
                        slot_name: None,
                        control_id: Some("Status".to_string()),
                        classes: Vec::new(),
                        params: BTreeMap::new(),
                        props: BTreeMap::from([(
                            "text".to_string(),
                            Value::String(text.to_string()),
                        )]),
                        layout: None,
                        bindings: Vec::new(),
                        style_overrides: Default::default(),
                        children: Vec::new(),
                    },
                }],
            }),
            components: BTreeMap::new(),
            stylesheets: Vec::new(),
        }
    }
}
