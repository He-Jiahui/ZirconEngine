use std::collections::BTreeMap;

use crate::ui::asset_editor::palette::{
    build_palette_entries, insert_palette_item_with_placement, PaletteInsertMode,
    UiAssetPaletteEntryKind, UiAssetPaletteInsertionPlacement,
};
use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::{
    component::UiHostCapabilitySet,
    template::{
        UiAssetDocument, UiAssetHeader, UiAssetImports, UiAssetKind, UiChildMount,
        UiComponentDefinition, UiNodeDefinition, UiNodeDefinitionKind, UiStyleDeclarationBlock,
        UiStyleScope,
    },
};

#[test]
fn ui_asset_editor_palette_uses_runtime_descriptor_native_entries() {
    let document = layout_document_with_local_component();
    let imports = BTreeMap::new();

    let entries = build_palette_entries(&document, &imports);
    let button = entries
        .iter()
        .find(|entry| entry.label == "Native / Button")
        .expect("Button should come from runtime descriptor palette metadata");
    let UiAssetPaletteEntryKind::Native {
        widget_type,
        default_node,
    } = &button.kind
    else {
        panic!("Button palette entry should be native");
    };
    assert_eq!(widget_type, "Button");
    assert_eq!(default_node.widget_type, "Button");
    assert_eq!(
        default_node.props.get("text").and_then(toml::Value::as_str),
        Some("Button")
    );

    let native_widget_types = entries
        .iter()
        .filter_map(|entry| match &entry.kind {
            UiAssetPaletteEntryKind::Native { widget_type, .. } => Some(widget_type.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let descriptor_widget_types = UiComponentDescriptorRegistry::editor_showcase()
        .palette_entries_for_host(&UiHostCapabilitySet::editor_authoring())
        .into_iter()
        .map(|entry| entry.component_id)
        .collect::<Vec<_>>();
    assert_eq!(native_widget_types, descriptor_widget_types);
}

#[test]
fn ui_asset_editor_palette_preserves_local_and_imported_reference_sources() {
    let document = layout_document_with_local_component();
    let mut imports = BTreeMap::new();
    let _ = imports.insert(
        "res://widgets/inspector.ui.toml#InspectorPanel".to_string(),
        imported_widget_document(),
    );

    let entries = build_palette_entries(&document, &imports);
    assert!(entries.iter().any(|entry| {
        entry.label == "Component / LocalPanel"
            && matches!(
                &entry.kind,
                UiAssetPaletteEntryKind::Component { component } if component == "LocalPanel"
            )
    }));
    assert!(entries.iter().any(|entry| {
        entry.label == "Reference / InspectorPanel"
            && matches!(
                &entry.kind,
                UiAssetPaletteEntryKind::Reference { component_ref }
                    if component_ref == "res://widgets/inspector.ui.toml#InspectorPanel"
            )
    }));
}

#[test]
fn ui_asset_editor_palette_instantiates_descriptor_default_node_templates() {
    let mut document = layout_document_with_local_component();
    let entries = build_palette_entries(&document, &BTreeMap::new());
    let button = entries
        .iter()
        .find(|entry| entry.label == "Native / Button")
        .expect("Button should be palette-visible");

    let inserted = insert_palette_item_with_placement(
        &mut document,
        "root",
        button,
        PaletteInsertMode::Child,
        &UiAssetPaletteInsertionPlacement::default(),
    )
    .expect("Button should insert into root container");
    let node = document.node(&inserted).expect("inserted node");
    assert_eq!(node.kind, UiNodeDefinitionKind::Native);
    assert_eq!(node.widget_type.as_deref(), Some("Button"));
    assert_eq!(
        node.props.get("text").and_then(toml::Value::as_str),
        Some("Button")
    );
}

#[test]
fn ui_asset_editor_palette_uses_runtime_descriptor_slots_for_native_children() {
    let mut document = layout_document_with_local_component();
    let entries = build_palette_entries(&document, &BTreeMap::new());
    let group = entries
        .iter()
        .find(|entry| entry.label == "Native / Group")
        .expect("Group should come from runtime descriptor palette metadata");
    let group_id = insert_palette_item_with_placement(
        &mut document,
        "root",
        group,
        PaletteInsertMode::Child,
        &UiAssetPaletteInsertionPlacement::default(),
    )
    .expect("Group should insert into root container");
    let button = entries
        .iter()
        .find(|entry| entry.label == "Native / Button")
        .expect("Button should come from runtime descriptor palette metadata");

    let button_id = insert_palette_item_with_placement(
        &mut document,
        &group_id,
        button,
        PaletteInsertMode::Child,
        &UiAssetPaletteInsertionPlacement::default(),
    )
    .expect("Group should accept children through its runtime descriptor slot");
    let mount = document
        .child_mount(&button_id)
        .expect("inserted button should have a child mount");
    assert_eq!(mount.mount.as_deref(), Some("content"));
}

fn layout_document_with_local_component() -> UiAssetDocument {
    let mut components = BTreeMap::new();
    let _ = components.insert(
        "LocalPanel".to_string(),
        UiComponentDefinition {
            root: native_node("local_panel_root", "Container"),
            style_scope: UiStyleScope::Closed,
            contract: Default::default(),
            params: BTreeMap::new(),
            slots: BTreeMap::new(),
        },
    );

    UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Layout,
            id: "test.palette".to_string(),
            version: 3,
            display_name: "Palette Test".to_string(),
        },
        imports: UiAssetImports::default(),
        tokens: BTreeMap::new(),
        root: Some(native_node("root", "Container")),
        components,
        stylesheets: Vec::new(),
    }
}

fn imported_widget_document() -> UiAssetDocument {
    let mut components = BTreeMap::new();
    let _ = components.insert(
        "InspectorPanel".to_string(),
        UiComponentDefinition {
            root: native_node("inspector_panel_root", "Container"),
            style_scope: UiStyleScope::Closed,
            contract: Default::default(),
            params: BTreeMap::new(),
            slots: BTreeMap::new(),
        },
    );
    UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Widget,
            id: "test.imported_widget".to_string(),
            version: 3,
            display_name: "Imported Widget".to_string(),
        },
        imports: UiAssetImports::default(),
        tokens: BTreeMap::new(),
        root: None,
        components,
        stylesheets: Vec::new(),
    }
}

fn native_node(node_id: &str, widget_type: &str) -> UiNodeDefinition {
    UiNodeDefinition {
        node_id: node_id.to_string(),
        kind: UiNodeDefinitionKind::Native,
        widget_type: Some(widget_type.to_string()),
        component: None,
        component_ref: None,
        component_api_version: None,
        slot_name: None,
        control_id: Some(widget_type.to_string()),
        classes: Vec::new(),
        params: BTreeMap::new(),
        props: BTreeMap::new(),
        layout: Some(BTreeMap::from([(
            "container".to_string(),
            table_value(&[("kind", toml::Value::String("Container".to_string()))]),
        )])),
        bindings: Vec::new(),
        style_overrides: UiStyleDeclarationBlock::default(),
        children: Vec::<UiChildMount>::new(),
    }
}

fn table_value(entries: &[(&str, toml::Value)]) -> toml::Value {
    toml::Value::Table(
        entries
            .iter()
            .map(|(key, value)| ((*key).to_string(), value.clone()))
            .collect(),
    )
}
