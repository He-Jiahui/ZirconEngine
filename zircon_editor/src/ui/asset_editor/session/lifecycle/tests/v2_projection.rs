use super::*;

#[test]
fn v2_projection_roundtrip_preserves_reference_component_and_named_mount() {
    let external_reference = UiNodeDefinition {
        node_id: "external_button".to_string(),
        kind: UiNodeDefinitionKind::Reference,
        component_ref: Some("res://ui/widgets/button.zui#ToolbarButton".to_string()),
        ..Default::default()
    };
    let local_component = UiNodeDefinition {
        node_id: "local_card".to_string(),
        kind: UiNodeDefinitionKind::Component,
        component: Some("Card".to_string()),
        ..Default::default()
    };
    let slot = UiNodeDefinition {
        node_id: "footer_slot".to_string(),
        kind: UiNodeDefinitionKind::Slot,
        slot_name: Some("footer".to_string()),
        ..Default::default()
    };
    let root = UiNodeDefinition {
        node_id: "root".to_string(),
        kind: UiNodeDefinitionKind::Native,
        widget_type: Some("VerticalBox".to_string()),
        children: vec![
            UiChildMount {
                mount: Some("footer".to_string()),
                node: external_reference,
                ..Default::default()
            },
            UiChildMount {
                node: local_component,
                ..Default::default()
            },
            UiChildMount {
                node: slot,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let component_root = UiNodeDefinition {
        node_id: "card_root".to_string(),
        kind: UiNodeDefinitionKind::Native,
        widget_type: Some("Panel".to_string()),
        ..Default::default()
    };
    let document = UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Layout,
            id: "ui.test.projection".to_string(),
            version: 1,
            display_name: "Projection".to_string(),
        },
        imports: Default::default(),
        tokens: Default::default(),
        root: Some(root),
        components: BTreeMap::from([(
            "Card".to_string(),
            UiComponentDefinition {
                root: component_root,
                ..Default::default()
            },
        )]),
        stylesheets: Vec::new(),
    };

    let v2 = legacy_projection_document_to_v2_document(&document, None)
        .expect("legacy authoring projection should convert to v2");
    assert_eq!(v2.asset.version, UI_V2_ASSET_SCHEMA_VERSION);
    assert_eq!(
        v2.nodes["external_button"].component,
        "res://ui/widgets/button.zui#ToolbarButton"
    );
    assert_eq!(v2.nodes["local_card"].component, "Card");
    assert_eq!(v2.nodes["footer_slot"].component, "Slot");
    assert_eq!(
        v2.nodes["footer_slot"].props["name"].as_str(),
        Some("footer")
    );
    assert_eq!(
        v2.nodes["root"].children[0].slot["name"].as_str(),
        Some("footer")
    );

    let projected = v2_document_to_legacy_projection_document(&v2)
        .expect("v2 authoring document should project back to the editor model");
    let children = &projected.root.expect("projected root").children;
    assert_eq!(children[0].mount.as_deref(), Some("footer"));
    assert_eq!(children[0].node.kind, UiNodeDefinitionKind::Reference);
    assert_eq!(
        children[0].node.component_ref.as_deref(),
        Some("res://ui/widgets/button.zui#ToolbarButton")
    );
    assert_eq!(children[1].node.kind, UiNodeDefinitionKind::Component);
    assert_eq!(children[1].node.component.as_deref(), Some("Card"));
    assert_eq!(children[2].node.kind, UiNodeDefinitionKind::Slot);
    assert_eq!(children[2].node.slot_name.as_deref(), Some("footer"));
}

#[test]
fn product_binding_fixture_projection_roundtrip_preserves_params_and_prior_state() {
    let source = r#"
[asset]
kind = "view"
id = "ui.test.param_projection"
version = 2

[root]
node = "root"

[nodes.root]
component = "BindingRow"
params = { label = "Before" }
state = { selected = true }
"#;
    let v2 = UiZuiAssetLoader::load_zui_str(source).expect("v2 param projection source");
    let mut projected = v2_document_to_legacy_projection_document(&v2)
        .expect("v2 params should project into the editor model");
    let root = projected.root.as_mut().expect("projected root");
    assert_eq!(
        root.params.get("label").and_then(toml::Value::as_str),
        Some("Before")
    );
    assert!(!root.params.contains_key("selected"));
    root.params.insert(
        "label".to_string(),
        toml::Value::String("After".to_string()),
    );

    let rebuilt = legacy_projection_document_to_v2_document(&projected, Some(&v2))
        .expect("editor projection should rebuild v2 params");

    assert_eq!(
        rebuilt.nodes["root"]
            .params
            .get("label")
            .and_then(toml::Value::as_str),
        Some("After")
    );
    assert_eq!(
        rebuilt.nodes["root"]
            .state
            .get("selected")
            .and_then(toml::Value::as_bool),
        Some(true)
    );
}

#[test]
fn designer_serialization_preserves_v2_repeat_and_node_slots() {
    let source = r#"
[asset]
kind = "view"
id = "ui.test.repeat_slot_projection"
version = 2

[root]
node = "root"

[nodes.root]
component = "VerticalBox"
params = { label = "Before" }

[[nodes.root.children]]
node = "virtual_rows"

[nodes.virtual_rows]
component = "VerticalBox"

[nodes.virtual_rows.repeat]
kind = "virtual_rows"
prototype = "row_template"
virtual_control_prefix = "Row"
authored_count = 1
node_path_namespace = "test"

[nodes.virtual_rows.slots]
heading = "Rows"

[[nodes.virtual_rows.children]]
node = "row_template"

[nodes.row_template]
component = "Label"
"#;
    let original = UiZuiAssetLoader::load_zui_str(source).expect("valid V2 view");
    let mut projected = v2_document_to_legacy_projection_document(&original)
        .expect("V2 view should project into Designer");
    projected
        .root
        .as_mut()
        .expect("projected root")
        .params
        .insert(
            "label".to_string(),
            toml::Value::String("After".to_string()),
        );

    let saved = serialize_v2_projection_document(&projected, Some(&original))
        .expect("Designer edit should serialize as V2");
    let reopened = UiZuiAssetLoader::load_zui_str(&saved).expect("saved V2 should reload");

    assert_eq!(
        reopened.nodes["root"].params["label"].as_str(),
        Some("After")
    );
    assert_eq!(
        reopened.nodes["virtual_rows"].repeat,
        original.nodes["virtual_rows"].repeat
    );
    assert_eq!(
        reopened.nodes["virtual_rows"].slots,
        original.nodes["virtual_rows"].slots
    );
}

#[test]
fn deleted_designer_node_does_not_inherit_prior_v2_nodes() {
    let source = r#"
[asset]
kind = "view"
id = "ui.test.deleted_projection_node"
version = 2

[root]
node = "root"

[nodes.root]
component = "VerticalBox"

[[nodes.root.children]]
node = "removed"

[nodes.removed]
component = "Label"

[nodes.removed.slots]
heading = "Old"
"#;
    let original = UiZuiAssetLoader::load_zui_str(source).expect("valid V2 view");
    let mut projected = v2_document_to_legacy_projection_document(&original)
        .expect("V2 view should project into Designer");
    projected.root.as_mut().expect("root").children.clear();

    let saved = serialize_v2_projection_document(&projected, Some(&original))
        .expect("deleted Designer child should serialize");
    let reopened = UiZuiAssetLoader::load_zui_str(&saved).expect("saved V2 should reload");

    assert!(!reopened.nodes.contains_key("removed"));
    assert!(reopened.nodes["root"].children.is_empty());
}

#[test]
fn component_projection_uses_component_root_without_view_root() {
    let document = component_projection_document();

    let v2 = legacy_projection_document_to_v2_document(&document, None)
        .expect("component projection should convert to v2");

    assert_eq!(v2.asset.kind, UiV2AssetKind::Component);
    assert!(
        v2.root.is_none(),
        "component assets must not declare a view root"
    );
    assert_eq!(v2.components["Button"].root, "button_root");
    assert!(v2.nodes.contains_key("button_root"));
}

#[test]
fn v2_serializer_rejects_component_assets_with_multiple_components() {
    let mut document = component_projection_document();
    let _ = document.components.insert(
        "SecondaryButton".to_string(),
        UiComponentDefinition {
            root: UiNodeDefinition {
                node_id: "secondary_button_root".to_string(),
                kind: UiNodeDefinitionKind::Native,
                widget_type: Some("Button".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
    );

    let error = serialize_v2_projection_document(&document, None)
        .expect_err("component serialization must enforce the v2 loader profile");

    assert!(matches!(
        error,
        UiAssetEditorSessionError::V2Asset(UiV2AssetError::InvalidDocument { detail, .. })
            if detail.contains("must declare exactly one component")
    ));
}

fn component_projection_document() -> UiAssetDocument {
    let component_root = UiNodeDefinition {
        node_id: "button_root".to_string(),
        kind: UiNodeDefinitionKind::Native,
        widget_type: Some("Button".to_string()),
        ..Default::default()
    };
    UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Widget,
            id: "ui.widgets.button".to_string(),
            version: 1,
            display_name: "Button".to_string(),
        },
        imports: Default::default(),
        tokens: Default::default(),
        root: Some(component_root.clone()),
        components: BTreeMap::from([(
            "Button".to_string(),
            UiComponentDefinition {
                root: component_root,
                ..Default::default()
            },
        )]),
        stylesheets: Vec::new(),
    }
}
