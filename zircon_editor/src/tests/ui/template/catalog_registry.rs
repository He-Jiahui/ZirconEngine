use super::support::EDITOR_HOST_WINDOW_ASSET_TOML;
use crate::ui::template::{
    EditorComponentCatalog, EditorComponentDescriptor, EditorTemplateRegistry,
    EditorTemplateRuntimeService,
};

#[test]
fn editor_component_catalog_registers_editor_only_composites() {
    let mut catalog = EditorComponentCatalog::default();
    catalog
        .register(EditorComponentDescriptor::new(
            "UiHostWindow",
            "ui.host_window",
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
fn editor_template_registry_instantiates_registered_documents() {
    let document =
        crate::tests::support::load_test_ui_asset(EDITOR_HOST_WINDOW_ASSET_TOML).unwrap();
    let template_service = EditorTemplateRuntimeService;
    let mut registry = EditorTemplateRegistry::default();
    template_service
        .register_asset_document(&mut registry, "ui.host_window", document)
        .unwrap();

    let instance = template_service
        .instantiate(&registry, "ui.host_window")
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
        .register_asset_document(&mut registry, "ui.host_window.asset", document)
        .unwrap();

    let instance = template_service
        .instantiate(&registry, "ui.host_window.asset")
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
