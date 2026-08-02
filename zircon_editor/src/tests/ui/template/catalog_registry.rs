use super::support::EDITOR_HOST_WINDOW_ASSET_TOML;
use crate::ui::template::{
    parse_editor_component_catalog_manifest, EditorComponentCatalog,
    EditorComponentCatalogManifestError, EditorComponentDescriptor, EditorComponentTier,
    EditorPropContract, EditorPropDefault, EditorPropLiteral, EditorSlotContract,
    EditorTemplateRegistry, EditorTemplateRuntimeService,
    EDITOR_COMPONENT_CATALOG_MANIFEST_FORMAT_VERSION,
};
use zircon_runtime_interface::ui::layout::UiSlotKind;

#[test]
fn editor_component_catalog_registers_editor_only_composites() {
    let mut catalog = EditorComponentCatalog::default();
    catalog
        .register(EditorComponentDescriptor::new(
            "UiHostWindow",
            "res://ui/editor/host/workbench_shell.zui",
            "UiHostWindow",
        ))
        .unwrap();
    catalog
        .register(EditorComponentDescriptor::new(
            "MenuBar",
            "workbench.menu_bar",
            "WorkbenchMenuBar",
        ))
        .unwrap();

    assert_eq!(
        catalog
            .descriptor("UiHostWindow")
            .unwrap()
            .binding_namespace,
        "UiHostWindow"
    );
    assert_eq!(catalog.descriptors().len(), 2);
}

#[test]
fn editor_component_catalog_records_tier_slot_and_token_backed_prop_contracts() {
    let descriptor = EditorComponentDescriptor::new(
        "WorkbenchToolbar",
        "res://ui/editor/components/workbench/shell/workbench_top_toolbar.zui",
        "WorkbenchToolbar",
    )
    .with_tier(EditorComponentTier::Composite)
    .with_slot(EditorSlotContract::new("actions", UiSlotKind::Linear).multiple(true))
    .with_token_prop("height", "dimension", "$control.height")
    .with_literal_prop("variant", "enum", "ghost");

    assert_eq!(descriptor.tier, EditorComponentTier::Composite);
    assert_eq!(descriptor.slots[0].name, "actions");
    assert_eq!(descriptor.slots[0].kind, UiSlotKind::Linear);
    assert!(descriptor.slots[0].multiple);
    assert_eq!(
        descriptor.props[0].default,
        EditorPropDefault::Token("$control.height".to_string())
    );
    assert_eq!(
        descriptor.props[0].default_token.as_deref(),
        Some("$control.height")
    );
    assert_eq!(
        descriptor.props[1].default,
        EditorPropDefault::Literal(EditorPropLiteral::Text("ghost".to_string()))
    );
}

#[test]
fn editor_prop_contract_migrates_the_legacy_default_token_field() {
    let prop: EditorPropContract = toml::from_str(
        r#"
name = "height"
value_type = "dimension"
default_token = "$editor.control.height.default"
"#,
    )
    .expect("legacy default_token props remain readable");

    assert_eq!(
        prop.default,
        EditorPropDefault::Token("$editor.control.height.default".to_string())
    );
    assert_eq!(
        prop.default_token.as_deref(),
        Some("$editor.control.height.default")
    );
}

#[test]
fn editor_prop_contract_uses_the_explicit_default_as_the_single_migration_source_of_truth() {
    let prop: EditorPropContract = toml::from_str(
        r#"
name = "variant"
value_type = "enum"
default = { literal = "ghost" }
default_token = "$editor.control.height.default"
"#,
    )
    .expect("an explicit modern default remains readable with stale legacy metadata");

    assert_eq!(
        prop.default,
        EditorPropDefault::Literal(EditorPropLiteral::Text("ghost".to_string()))
    );
    assert_eq!(prop.default_token, None);
}

#[test]
fn editor_prop_contract_explicit_none_clears_a_stale_legacy_default_token() {
    let prop: EditorPropContract = toml::from_str(
        r#"
name = "icon"
value_type = "icon_ref"
default = "none"
default_token = "$editor.icon.default"
"#,
    )
    .expect("an explicit None default remains readable with stale legacy metadata");

    assert_eq!(prop.default, EditorPropDefault::None);
    assert_eq!(prop.default_token, None);
}

#[test]
fn editor_prop_contract_preserves_native_literal_value_types() {
    let boolean: EditorPropContract = toml::from_str(
        r#"
name = "checked"
value_type = "boolean"
default = { literal = false }
"#,
    )
    .expect("boolean literal defaults should deserialize without string coercion");
    assert_eq!(
        boolean.default,
        EditorPropDefault::Literal(EditorPropLiteral::Boolean(false))
    );

    let floating: EditorPropContract = toml::from_str(
        r#"
name = "value"
value_type = "number"
default = { literal = 50.0 }
"#,
    )
    .expect("floating literal defaults should deserialize without string coercion");
    assert_eq!(
        floating.default,
        EditorPropDefault::Literal(EditorPropLiteral::Float(50.0))
    );

    let integer: EditorPropContract = toml::from_str(
        r#"
name = "columns"
value_type = "integer"
default = { literal = 3 }
"#,
    )
    .expect("integer literal defaults should deserialize without float coercion");
    assert_eq!(
        integer.default,
        EditorPropDefault::Literal(EditorPropLiteral::Integer(3))
    );

    let menu_items: EditorPropContract = toml::from_str(
        r#"
name = "menu_items"
value_type = "text_list"
default = { literal = ["New", "Open"] }
"#,
    )
    .expect("text-list literal defaults should preserve individual items");
    assert_eq!(
        menu_items.default,
        EditorPropDefault::Literal(EditorPropLiteral::TextList(vec![
            "New".to_string(),
            "Open".to_string(),
        ]))
    );
}

#[test]
fn editor_component_catalog_deserializes_legacy_descriptors_without_contract_metadata() {
    let descriptor: EditorComponentDescriptor = toml::from_str(
        r#"
component_id = "LegacyToolbar"
document_id = "res://ui/editor/legacy_toolbar.zui"
binding_namespace = "LegacyToolbar"
"#,
    )
    .expect("legacy descriptor remains readable after catalog metadata expansion");

    assert_eq!(descriptor.tier, EditorComponentTier::Composite);
    assert!(descriptor.slots.is_empty());
    assert!(descriptor.props.is_empty());
}

#[test]
fn editor_component_catalog_manifest_parses_versioned_component_contracts() {
    let descriptors = parse_editor_component_catalog_manifest(
        r#"
version = 1

[[components]]
component_id = "PropertyField"
document_id = "res://ui/editor/components/property_field.zui"
binding_namespace = "PropertyField"
tier = "composite"

[[components.slots]]
name = "editor"
kind = "linear"
required = true
accepts = ["Dropdown", "Field", "Slider", "Toggle"]

[[components.props]]
name = "height"
value_type = "dimension"
default = { token = "$editor.control.height.default" }
"#,
    )
    .expect("versioned catalog asset should deserialize");

    assert_eq!(descriptors.len(), 1);
    assert_eq!(descriptors[0].tier, EditorComponentTier::Composite);
    assert_eq!(descriptors[0].slots[0].kind, UiSlotKind::Linear);
    assert!(descriptors[0].slots[0].required);
    assert!(descriptors[0].slots[0].accepts.contains("Dropdown"));
    assert_eq!(
        descriptors[0].props[0].default,
        EditorPropDefault::Token("$editor.control.height.default".to_string())
    );
}

#[test]
fn editor_component_catalog_manifest_rejects_unknown_versions_and_duplicate_identities() {
    let version_error = parse_editor_component_catalog_manifest(
        r#"
version = 2
components = []
"#,
    )
    .expect_err("future catalog versions must be rejected explicitly");
    assert_eq!(
        version_error,
        EditorComponentCatalogManifestError::UnsupportedVersion {
            version: 2,
            expected: EDITOR_COMPONENT_CATALOG_MANIFEST_FORMAT_VERSION,
        }
    );

    let duplicate_error = parse_editor_component_catalog_manifest(
        r#"
version = 1

[[components]]
component_id = "Button"
document_id = "res://ui/editor/components/button.zui"
binding_namespace = "Button"

[[components]]
component_id = "Button"
document_id = "res://ui/editor/components/button_secondary.zui"
binding_namespace = "ButtonSecondary"
"#,
    )
    .expect_err("component catalog identities must remain unique");
    assert_eq!(
        duplicate_error,
        EditorComponentCatalogManifestError::DuplicateComponent {
            component_id: "Button".to_string(),
        }
    );
}

#[test]
fn editor_component_catalog_manifest_rejects_non_zui_component_documents() {
    let error = parse_editor_component_catalog_manifest(
        r#"
version = 1

[[components]]
component_id = "LegacyToolbar"
document_id = "res://ui/editor/components/legacy_toolbar.v2.ui.toml"
binding_namespace = "LegacyToolbar"
"#,
    )
    .expect_err("typed component metadata must only reference current .zui documents");

    assert_eq!(
        error,
        EditorComponentCatalogManifestError::InvalidDocumentReference {
            component_id: "LegacyToolbar".to_string(),
            document_id: "res://ui/editor/components/legacy_toolbar.v2.ui.toml".to_string(),
        }
    );
}

#[test]
fn editor_component_catalog_manifest_rejects_documents_outside_the_editor_asset_root() {
    let error = parse_editor_component_catalog_manifest(
        r#"
version = 1

[[components]]
component_id = "ExternalToolbar"
document_id = "res://ui/runtime/components/toolbar.zui"
binding_namespace = "ExternalToolbar"
"#,
    )
    .expect_err("builtin editor component metadata must not cross into another asset root");

    assert_eq!(
        error,
        EditorComponentCatalogManifestError::InvalidDocumentReference {
            component_id: "ExternalToolbar".to_string(),
            document_id: "res://ui/runtime/components/toolbar.zui".to_string(),
        }
    );
}

#[test]
fn editor_component_catalog_manifest_rejects_unknown_schema_fields() {
    let error = parse_editor_component_catalog_manifest(
        r#"
version = 1

[[components]]
component_id = "Toolbar"
document_id = "res://ui/editor/components/toolbar.zui"
binding_namespace = "Toolbar"
tier_typo = "primitive"
"#,
    )
    .expect_err("catalog schema typos must not become silently ignored metadata");

    assert!(matches!(
        error,
        EditorComponentCatalogManifestError::Parse { detail }
            if detail.contains("unknown field `tier_typo`")
    ));
}

#[test]
fn editor_component_catalog_manifest_rejects_malformed_token_defaults() {
    let error = parse_editor_component_catalog_manifest(
        r#"
version = 1

[[components]]
component_id = "Toolbar"
document_id = "res://ui/editor/components/toolbar.zui"
binding_namespace = "Toolbar"

[[components.props]]
name = "height"
value_type = "dimension"
default = { token = "editor.control.height.default" }
"#,
    )
    .expect_err("token-backed defaults must retain the runtime token syntax");

    assert_eq!(
        error,
        EditorComponentCatalogManifestError::InvalidTokenDefault {
            component_id: "Toolbar".to_string(),
            property: "height".to_string(),
            token: "editor.control.height.default".to_string(),
        }
    );
}

#[test]
fn editor_component_catalog_manifest_rejects_duplicate_slot_and_property_contract_names() {
    let duplicate_slot = parse_editor_component_catalog_manifest(
        r#"
version = 1

[[components]]
component_id = "Panel"
document_id = "res://ui/editor/components/panel.zui"
binding_namespace = "Panel"

[[components.slots]]
name = "content"
kind = "linear"

[[components.slots]]
name = "content"
kind = "linear"
"#,
    )
    .expect_err("one component cannot expose the same slot contract twice");
    assert_eq!(
        duplicate_slot,
        EditorComponentCatalogManifestError::DuplicateSlot {
            component_id: "Panel".to_string(),
            slot: "content".to_string(),
        }
    );

    let duplicate_property = parse_editor_component_catalog_manifest(
        r#"
version = 1

[[components]]
component_id = "Panel"
document_id = "res://ui/editor/components/panel.zui"
binding_namespace = "Panel"

[[components.props]]
name = "height"
value_type = "dimension"

[[components.props]]
name = "height"
value_type = "dimension"
"#,
    )
    .expect_err("one component cannot expose the same prop contract twice");
    assert_eq!(
        duplicate_property,
        EditorComponentCatalogManifestError::DuplicateProperty {
            component_id: "Panel".to_string(),
            property: "height".to_string(),
        }
    );
}

#[test]
fn editor_template_registry_instantiates_registered_documents() {
    let document =
        crate::tests::support::load_test_ui_asset(EDITOR_HOST_WINDOW_ASSET_TOML).unwrap();
    let template_service = EditorTemplateRuntimeService;
    let mut registry = EditorTemplateRegistry::default();
    template_service
        .register_asset_document(
            &mut registry,
            "res://ui/editor/host/workbench_shell.zui",
            document,
        )
        .unwrap();

    let instance = template_service
        .instantiate(&registry, "res://ui/editor/host/workbench_shell.zui")
        .unwrap();
    assert_eq!(instance.root.component.as_deref(), Some("UiHostWindow"));
    assert_eq!(
        instance.root.children[0].component.as_deref(),
        Some("UiHostToolbar")
    );
    assert_eq!(
        instance.root.children[1].component.as_deref(),
        Some("StatusBar")
    );
}

#[test]
fn editor_template_registry_instantiates_registered_asset_documents() {
    let document =
        crate::tests::support::load_test_ui_asset(EDITOR_HOST_WINDOW_ASSET_TOML).unwrap();
    let template_service = EditorTemplateRuntimeService;
    let mut registry = EditorTemplateRegistry::default();
    template_service
        .register_asset_document(&mut registry, "test://ui/host-window-asset", document)
        .unwrap();

    let instance = template_service
        .instantiate(&registry, "test://ui/host-window-asset")
        .unwrap();
    assert_eq!(instance.root.component.as_deref(), Some("UiHostWindow"));
    assert_eq!(
        instance.root.children[0].component.as_deref(),
        Some("UiHostToolbar")
    );
    assert_eq!(
        instance.root.children[1].component.as_deref(),
        Some("StatusBar")
    );
}
