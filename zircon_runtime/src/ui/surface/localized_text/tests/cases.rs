use super::*;
use std::collections::BTreeMap;
use zircon_runtime_interface::ui::{
    binding::UiBindingSourceKind,
    event_ui::{UiNodePath, UiTreeId},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

fn localized_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("localized-text-contract"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/title")).with_template_metadata(
            UiTemplateNodeMetadata {
                component: "Text".to_owned(),
                attributes: BTreeMap::from([(
                    "text".to_owned(),
                    toml::Value::Table(toml::map::Map::from_iter([
                        (
                            "text_key".to_owned(),
                            toml::Value::String("title".to_owned()),
                        ),
                        ("table".to_owned(), toml::Value::String("editor".to_owned())),
                    ])),
                )]),
                ..Default::default()
            },
        ),
    );
    surface
}
fn resolve(surface: &mut UiSurface, text: &str) {
    surface
        .synchronize_localized_text(|reference| {
            assert_eq!(reference.key, "title");
            Ok::<_, std::io::Error>(text.to_owned())
        })
        .unwrap();
}

#[test]
fn resolved_payload_retains_source_identity_and_explicit_empty_write_unbinds_it() {
    let mut surface = localized_surface();
    resolve(&mut surface, "Localized heading");
    let metadata = surface
        .tree
        .node(UiNodeId::new(1))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(
        metadata.attributes["text"].as_str(),
        Some("Localized heading")
    );
    assert_eq!(metadata.localized_text_references["text"].key, "title");
    let encoded = serde_json::to_string(metadata).unwrap();
    let restored: UiTemplateNodeMetadata = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        restored.localized_text_references,
        metadata.localized_text_references
    );
    resolve(&mut surface, "Changed locale heading");
    surface
        .mutate_property(UiPropertyMutationRequest::new(
            UiNodeId::new(1),
            "text",
            UiValue::String(String::new()),
        ))
        .unwrap();
    resolve(&mut surface, "Another locale heading");
    let metadata = surface
        .tree
        .node(UiNodeId::new(1))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["text"].as_str(), Some(""));
    assert!(metadata.localized_text_references.is_empty());
}

#[test]
fn same_value_explicit_write_and_batch_write_also_end_the_source_binding() {
    let mut surface = localized_surface();
    resolve(&mut surface, "Heading");
    let report = surface
        .mutate_property(UiPropertyMutationRequest::new(
            UiNodeId::new(1),
            "text",
            UiValue::String("Heading".to_owned()),
        ))
        .unwrap();
    assert_eq!(report.status, UiPropertyMutationStatus::Unchanged);
    resolve(&mut surface, "Changed locale");
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes["text"]
            .as_str(),
        Some("Heading")
    );
    let mut surface = localized_surface();
    resolve(&mut surface, "Heading");
    super::super::property_mutation::mutate_tree_metadata_properties(
        &mut surface.tree,
        UiNodeId::new(1),
        [("text", UiValue::String("Camera".to_owned()))],
        UiBindingSourceKind::RuntimeState,
    )
    .unwrap();
    resolve(&mut surface, "Changed locale");
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(1))
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes["text"]
            .as_str(),
        Some("Camera")
    );
}
