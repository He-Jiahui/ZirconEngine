use super::support::EDITOR_WORKBENCH_ASSET_TOML;
use crate::ui::template::{
    EditorComponentCatalog, EditorComponentDescriptor, EditorTemplateRegistry,
};

#[test]
fn editor_component_catalog_registers_editor_only_composites() {
    let mut catalog = EditorComponentCatalog::default();
    catalog
        .register(EditorComponentDescriptor::new(
            "WorkbenchShell",
            "workbench.shell",
            "WorkbenchShell",
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
            .descriptor("WorkbenchShell")
            .unwrap()
            .binding_namespace,
        "WorkbenchShell"
    );
    assert_eq!(catalog.descriptors().len(), 2);
}

#[test]
fn editor_template_registry_instantiates_registered_documents() {
    let document = crate::tests::support::load_test_ui_asset(EDITOR_WORKBENCH_ASSET_TOML).unwrap();
    let mut registry = EditorTemplateRegistry::default();
    registry
        .register_asset_document("workbench.shell", document)
        .unwrap();

    let instance = registry.instantiate("workbench.shell").unwrap();
    assert_eq!(instance.root.component.as_deref(), Some("WorkbenchShell"));
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
    let document = crate::tests::support::load_test_ui_asset(EDITOR_WORKBENCH_ASSET_TOML).unwrap();
    let mut registry = EditorTemplateRegistry::default();
    registry
        .register_asset_document("workbench.shell.asset", document)
        .unwrap();

    let instance = registry.instantiate("workbench.shell.asset").unwrap();
    assert_eq!(instance.root.component.as_deref(), Some("WorkbenchShell"));
    assert_eq!(
        instance.root.children[0].component.as_deref(),
        Some("UiHostToolbar")
    );
    assert_eq!(
        instance.root.children[1].component.as_deref(),
        Some("StatusBar")
    );
}
