use super::*;
use zircon_runtime_interface::ui::layout::UiSlotKind;

#[test]
fn ui_v2_composite_component_patches_root_props_and_fills_slots() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/composite.v2.ui"
version = 2

[root]
node = "root"

[components.Card]
root = "card_root"
default_classes = ["material-card"]
slots = { content = { kind = "linear", accepts = ["Text"] } }

[nodes.root]
component = "Card"
control_id = "AssetCard"
classes = ["dense"]
props = { surface_variant = "outlined" }

[[nodes.root.children]]
node = "card_body"
slot = { name = "content" }

[nodes.card_root]
component = "VerticalGroup"
control_id = "CardPrototypeRoot"
props = { surface_variant = "filled" }

[[nodes.card_root.children]]
node = "card_title"

[[nodes.card_root.children]]
node = "card_content_slot"

[nodes.card_title]
component = "Text"
control_id = "CardTitle"
props = { text = "Prototype" }

[nodes.card_content_slot]
component = "Slot"
props = { name = "content" }

[nodes.card_body]
component = "Text"
control_id = "CardBody"
props = { text = "Instanced body" }
"#,
    )
    .unwrap();

    let content_slot = &document.components["Card"].slots["content"];
    assert_eq!(content_slot.kind, Some(UiSlotKind::Linear));
    assert!(content_slot.accepts_component("Text"));
    assert!(!content_slot.accepts_component("Button"));

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let root = compiled
        .arena
        .node(compiled.arena.root.expect("expanded root"))
        .unwrap();

    assert_eq!(compiled.arena.node_count(), 3);
    assert_eq!(root.component, "VerticalGroup");
    assert_eq!(root.control_id.as_deref(), Some("AssetCard"));
    assert!(root.classes.iter().any(|class| class == "material-card"));
    assert!(root.classes.iter().any(|class| class == "dense"));
    assert_eq!(
        root.props.get("surface_variant").and_then(Value::as_str),
        Some("outlined")
    );
    assert!(compiled
        .arena
        .nodes
        .iter()
        .any(|node| node.control_id.as_deref() == Some("CardBody")));
}

#[test]
fn ui_v2_composite_component_forwards_default_slot_children() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/default_slot.v2.ui"
version = 2

[root]
node = "root"

[components.TabStrip]
root = "strip_root"
slots = { default = { multiple = true } }

[nodes.root]
component = "TabStrip"
control_id = "RuntimeTabStrip"
children = [{ node = "tab_a" }, { node = "tab_b" }]

[nodes.strip_root]
component = "HorizontalGroup"
control_id = "TabStripRoot"
children = [{ node = "default_slot" }]

[nodes.default_slot]
component = "Slot"
props = { name = "default" }

[nodes.tab_a]
component = "ToggleButton"
control_id = "RuntimeTabA"
props = { text = "A", selected = true }

[nodes.tab_b]
component = "ToggleButton"
control_id = "RuntimeTabB"
props = { text = "B" }
"#,
    )
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let root = compiled
        .arena
        .node(compiled.arena.root.expect("expanded root"))
        .unwrap();

    assert_eq!(root.control_id.as_deref(), Some("RuntimeTabStrip"));
    assert_eq!(root.children.len(), 2);
    assert!(compiled
        .arena
        .nodes
        .iter()
        .any(|node| node.control_id.as_deref() == Some("RuntimeTabA")));
    assert!(compiled
        .arena
        .nodes
        .iter()
        .any(|node| node.control_id.as_deref() == Some("RuntimeTabB")));
}

#[test]
fn ui_v2_composite_component_preserves_slot_placeholder_layout_on_filled_child() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/property_row_slot_layout.v2.ui"
version = 2

[root]
node = "root"

[components.PropertyEditorRow]
root = "row_root"
slots = { value = { multiple = false } }

[nodes.root]
component = "PropertyEditorRow"
layout = { width = { stretch = "Stretch" }, height = { min = 28.0, preferred = 30.0, max = 32.0, stretch = "Fixed" } }
children = [{ node = "field", slot = { name = "value" } }]

[nodes.row_root]
component = "PropertyRow"
layout = { container = { kind = "HorizontalBox", gap = 4.0 }, width = { stretch = "Stretch" }, height = { min = 28.0, preferred = 30.0, max = 32.0, stretch = "Fixed" } }
children = [{ node = "name" }, { node = "value_slot" }]

[nodes.name]
component = "Container"
layout = { width = { min = 60.0, preferred = 105.0, max = 105.0, stretch = "Fixed" } }

[nodes.value_slot]
component = "Slot"
props = { name = "value" }
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" } }

[nodes.field]
component = "TextField"
control_id = "PropertyValueField"
"#,
    )
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let root = compiled
        .arena
        .node(compiled.arena.root.expect("expanded row root"))
        .unwrap();
    let value_mount = root
        .children
        .iter()
        .find(|child| {
            compiled
                .arena
                .node(child.child)
                .is_some_and(|node| node.control_id.as_deref() == Some("PropertyValueField"))
        })
        .expect("filled value child should replace its slot placeholder");

    assert_eq!(
        root.layout
            .as_ref()
            .and_then(|layout| layout.get("container"))
            .and_then(|container| container.get("kind"))
            .and_then(Value::as_str),
        Some("HorizontalBox")
    );
    assert_eq!(
        value_mount
            .slot
            .get("layout")
            .and_then(|layout| layout.get("width"))
            .and_then(|width| width.get("stretch"))
            .and_then(Value::as_str),
        Some("Stretch")
    );

    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.property_row_slot_layout"),
        &document,
        &compiled,
    )
    .unwrap();
    surface.compute_layout(UiSize::new(260.0, 30.0)).unwrap();

    let row_frame = surface
        .arranged_tree
        .get(surface.tree.roots[0])
        .expect("expanded property row root should be arranged")
        .frame;
    let field_frame = surface
        .arranged_tree
        .get(node_id_by_control_id(&surface, "PropertyValueField"))
        .expect("filled property editor should be arranged")
        .frame;

    assert_eq!(field_frame.x, row_frame.x + 109.0);
    assert_eq!(field_frame.width, row_frame.width - 109.0);
}

#[test]
fn ui_v2_composite_component_validates_declared_slots() {
    let mut document = v2_document("asset://ui/tests/slot_validation.v2.ui", "root");
    document.components.insert(
        "Card".to_string(),
        zircon_runtime_interface::ui::v2::UiV2ComponentDefinition {
            root: "card_root".to_string(),
            slots: BTreeMap::from([(
                "content".to_string(),
                UiNamedSlotSchema {
                    required: true,
                    multiple: false,
                    ..UiNamedSlotSchema::default()
                },
            )]),
            ..Default::default()
        },
    );
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Card".to_string(),
            children: vec![
                UiV2ChildMount {
                    node: "body_a".to_string(),
                    slot: BTreeMap::from([(
                        "name".to_string(),
                        Value::String("content".to_string()),
                    )]),
                },
                UiV2ChildMount {
                    node: "body_b".to_string(),
                    slot: BTreeMap::from([(
                        "name".to_string(),
                        Value::String("content".to_string()),
                    )]),
                },
            ],
            ..Default::default()
        },
    );
    document.nodes.insert(
        "card_root".to_string(),
        UiV2NodeDefinition {
            component: "Slot".to_string(),
            props: BTreeMap::from([("name".to_string(), Value::String("content".to_string()))]),
            ..Default::default()
        },
    );
    document.nodes.insert(
        "body_a".to_string(),
        UiV2NodeDefinition {
            component: "Text".to_string(),
            ..Default::default()
        },
    );
    document.nodes.insert(
        "body_b".to_string(),
        UiV2NodeDefinition {
            component: "Text".to_string(),
            ..Default::default()
        },
    );

    let error = UiV2DocumentCompiler::compile(&document).unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::SlotDoesNotAcceptMultiple { slot_name, .. } if slot_name == "content"
    ));
}

#[test]
fn ui_v2_composite_component_rejects_slot_fills_outside_the_declared_accept_set() {
    let mut document = v2_document("asset://ui/tests/slot_accepts.v2.ui", "root");
    document.components.insert(
        "Card".to_string(),
        zircon_runtime_interface::ui::v2::UiV2ComponentDefinition {
            root: "card_root".to_string(),
            slots: BTreeMap::from([(
                "content".to_string(),
                UiNamedSlotSchema {
                    accepts: ["Text".to_string()].into_iter().collect(),
                    ..UiNamedSlotSchema::default()
                },
            )]),
            ..Default::default()
        },
    );
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Card".to_string(),
            children: vec![UiV2ChildMount {
                node: "body".to_string(),
                slot: BTreeMap::from([("name".to_string(), Value::String("content".to_string()))]),
            }],
            ..Default::default()
        },
    );
    document.nodes.insert(
        "card_root".to_string(),
        UiV2NodeDefinition {
            component: "Slot".to_string(),
            props: BTreeMap::from([("name".to_string(), Value::String("content".to_string()))]),
            ..Default::default()
        },
    );
    document.nodes.insert(
        "body".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            ..Default::default()
        },
    );

    let error = UiV2DocumentCompiler::compile(&document).expect_err("Button is not accepted");

    assert!(matches!(
        error,
        UiV2AssetError::SlotDoesNotAcceptComponent {
            slot_name,
            child_component,
            ..
        } if slot_name == "content" && child_component == "Button"
    ));
}

#[test]
fn ui_v2_composite_component_accepts_explicit_component_reference_slot_fills() {
    let external_component = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/components/slot_panel.v2.ui"
version = 2

[components.Panel]
root = "panel_root"

[nodes.panel_root]
component = "Container"
control_id = "PrototypePanel"
"#,
    )
    .expect("external panel prototype should parse");
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/explicit_slot_component_reference.v2.ui"
version = 2

[root]
node = "root"

[components.Card]
root = "card_root"
slots = { content = { accepts = ["Panel"] } }

[nodes.root]
component = "Card"
children = [{ node = "external_panel", slot = { name = "content" } }]

[nodes.card_root]
component = "VerticalGroup"
children = [{ node = "content_slot" }]

[nodes.content_slot]
component = "Slot"
props = { name = "content" }

[nodes.external_panel]
component = "asset://ui/components/slot_panel.v2.ui#Panel"
control_id = "ExternalPanel"
"#,
    )
    .expect("explicit component-reference slot fixture should parse");
    let mut store = UiV2PrototypeStore::new();
    store.insert(external_component);

    let compiled = UiV2DocumentCompiler::compile_with_prototype_store(&document, &store)
        .expect("the slot contract should compare the named component identity");
    assert!(compiled
        .arena
        .nodes
        .iter()
        .any(|node| node.control_id.as_deref() == Some("ExternalPanel")
            && node.component == "Container"));
}

#[test]
fn ui_v2_composite_component_can_be_loaded_from_prototype_store() {
    let component = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/components/material_button.v2.ui"
version = 2

[components.MaterialButton]
root = "button_root"

[nodes.button_root]
component = "Button"
control_id = "PrototypeButton"
props = { text = "Prototype" }
"#,
    )
    .unwrap();
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/imported_component.v2.ui"
version = 2

[imports]
widgets = ["asset://ui/components/material_button.v2.ui#MaterialButton"]

[root]
node = "root"

[nodes.root]
component = "MaterialButton"
control_id = "ApplyDraft"
props = { text = "Apply Draft" }
"#,
    )
    .unwrap();
    let mut store = UiV2PrototypeStore::new();
    store.insert(component);

    let compiled = UiV2DocumentCompiler::compile_with_prototype_store(&document, &store).unwrap();
    let root = compiled
        .arena
        .node(compiled.arena.root.expect("expanded root"))
        .unwrap();

    assert_eq!(compiled.arena.node_count(), 1);
    assert_eq!(root.component, "Button");
    assert_eq!(root.control_id.as_deref(), Some("ApplyDraft"));
    assert_eq!(
        root.props.get("text").and_then(Value::as_str),
        Some("Apply Draft")
    );
}

#[test]
fn ui_v2_prototype_store_builder_tracks_whole_asset_widget_imports() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/whole_import_owner.v2.ui"
version = 2

[imports]
widgets = ["asset://ui/components/material_button.v2.ui"]

[root]
node = "root"

[nodes.root]
component = "Label"
"#,
    )
    .expect("whole-asset widget import fixture should parse");
    let mut builder = UiV2PrototypeStoreBuilder::new();
    let _ = builder.insert(document);

    let error = builder
        .build()
        .expect_err("whole-asset imports must be loaded before the store is built");
    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { asset_id, detail }
            if asset_id == "asset://ui/components/material_button.v2.ui"
                && detail.contains("declared UI v2 import is not loaded")
    ));
}

#[test]
fn ui_v2_prototype_store_builder_rejects_widget_imports_with_multiple_fragments() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/ambiguous_import_owner.v2.ui"
version = 2

[imports]
widgets = ["asset://ui/components/material_button.v2.ui#MaterialButton#Unexpected"]

[root]
node = "root"

[nodes.root]
component = "Label"
"#,
    )
    .expect("ambiguous widget import fixture should parse before contract validation");
    let mut builder = UiV2PrototypeStoreBuilder::new();
    let _ = builder.insert(document);

    let error = builder
        .build()
        .expect_err("multiple component fragments must fail before prototype lookup");
    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { asset_id, detail }
            if asset_id == "asset://ui/tests/ambiguous_import_owner.v2.ui"
                && detail.contains("exactly one non-empty #Component suffix")
    ));
}

#[test]
fn ui_v2_imported_component_instance_style_patches_expanded_root() {
    let component = UiV2AssetLoader::load_toml_str(
        r##"
[asset]
kind = "component"
id = "asset://ui/components/style_label.zui"
version = 2

[components.StyleLabel]
root = "label_root"

[nodes.label_root]
component = "Label"
control_id = "PrototypeLabel"
props = { text = "Prototype", foreground_color = "#d8e3e7" }
"##,
    )
    .unwrap();
    let document = UiV2AssetLoader::load_toml_str(
        r##"
[asset]
kind = "view"
id = "asset://ui/tests/imported_component_instance_style.v2.ui"
version = 2

[imports]
widgets = ["asset://ui/components/style_label.zui#StyleLabel"]

[root]
node = "root"

[nodes.root]
component = "StyleLabel"
control_id = "RuntimeLabel"
style = { self = { foreground_color = "#ef493f", text_tone = "error" } }

[[stylesheets]]
id = "strict_label"

[[stylesheets.rules]]
selector = "Label"
set = { self = { foreground_color = "#d8e3e7", text_tone = "primary" } }
"##,
    )
    .unwrap();
    let mut store = UiV2PrototypeStore::new();
    store.insert(component);

    let compiled = UiV2DocumentCompiler::compile_with_prototype_store(&document, &store).unwrap();
    let root = compiled
        .arena
        .node(compiled.arena.root.expect("expanded root"))
        .unwrap();

    assert_eq!(root.component, "Label");
    assert_eq!(root.control_id.as_deref(), Some("RuntimeLabel"));
    assert_eq!(
        root.style
            .self_values
            .get("foreground_color")
            .and_then(Value::as_str),
        Some("#ef493f")
    );
    assert_eq!(
        root.style
            .self_values
            .get("text_tone")
            .and_then(Value::as_str),
        Some("error")
    );

    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.imported_component_instance_style"),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = node_id_by_control_id(&surface, "RuntimeLabel");
    let metadata = surface
        .tree
        .nodes
        .get(&node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(
        metadata
            .style_overrides
            .get("foreground_color")
            .and_then(Value::as_str),
        Some("#ef493f")
    );
    assert_eq!(
        metadata
            .style_overrides
            .get("text_tone")
            .and_then(Value::as_str),
        Some("error")
    );
}
