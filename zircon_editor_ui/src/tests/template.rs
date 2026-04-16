use crate::{
    EditorComponentCatalog, EditorComponentDescriptor, EditorTemplateAdapter, EditorTemplateError,
    EditorTemplateRegistry, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind,
};
use std::path::PathBuf;
use zircon_ui::{UiEventKind, UiTemplateLoader};

const EDITOR_WORKBENCH_TEMPLATE_TOML: &str = r#"
version = 1

[root]
template = "WorkbenchShell"
slots = { menu_bar = [{ template = "MenuBar" }], status_bar = [{ component = "StatusBar", control_id = "StatusBarRoot" }] }

[components.WorkbenchShell]
slots = { menu_bar = { required = true }, status_bar = { required = true } }
root = { component = "WorkbenchShell", children = [{ slot = "menu_bar" }, { slot = "status_bar" }] }

[components.MenuBar]
root = { component = "UiHostToolbar", children = [
    { component = "UiHostIconButton", control_id = "OpenProject", bindings = [{ id = "WorkbenchMenuBar/OpenProject", event = "Click", route = "MenuAction.OpenProject" }] },
    { component = "UiHostIconButton", control_id = "SaveProject", bindings = [{ id = "WorkbenchMenuBar/SaveProject", event = "Click", route = "MenuAction.SaveProject" }] }
] }
"#;

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
    let document = UiTemplateLoader::load_toml_str(EDITOR_WORKBENCH_TEMPLATE_TOML).unwrap();
    let mut registry = EditorTemplateRegistry::default();
    registry
        .register_document("workbench.shell", document)
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
fn editor_template_adapter_resolves_stable_binding_ids_to_typed_editor_bindings() {
    let document = UiTemplateLoader::load_toml_str(EDITOR_WORKBENCH_TEMPLATE_TOML).unwrap();
    let mut registry = EditorTemplateRegistry::default();
    registry
        .register_document("workbench.shell", document)
        .unwrap();
    let instance = registry.instantiate("workbench.shell").unwrap();

    let mut adapter = EditorTemplateAdapter::default();
    adapter
        .register_binding(
            "WorkbenchMenuBar/OpenProject",
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "OpenProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("OpenProject"),
            ),
        )
        .unwrap();
    adapter
        .register_binding(
            "WorkbenchMenuBar/SaveProject",
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "SaveProject",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("SaveProject"),
            ),
        )
        .unwrap();

    let bindings = adapter.resolve_instance_bindings(&instance).unwrap();
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].path().event_kind, UiEventKind::Click);
    assert_eq!(bindings[0].path().view_id, "WorkbenchMenuBar");
    assert_eq!(bindings[0].path().control_id, "OpenProject");
    assert_eq!(bindings[1].path().control_id, "SaveProject");
}

#[test]
fn editor_template_adapter_rejects_missing_binding_resolution() {
    let document = UiTemplateLoader::load_toml_str(EDITOR_WORKBENCH_TEMPLATE_TOML).unwrap();
    let mut registry = EditorTemplateRegistry::default();
    registry
        .register_document("workbench.shell", document)
        .unwrap();
    let instance = registry.instantiate("workbench.shell").unwrap();

    let error = EditorTemplateAdapter::default()
        .resolve_instance_bindings(&instance)
        .unwrap_err();
    assert_eq!(
        error,
        EditorTemplateError::MissingBinding {
            binding_id: "WorkbenchMenuBar/OpenProject".to_string(),
        }
    );
}

#[test]
fn editor_repository_workbench_template_file_loads_and_instantiates() {
    let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../zircon_editor/ui/templates/workbench_shell.toml");
    let document = UiTemplateLoader::load_toml_file(&template_path).unwrap();

    let mut registry = EditorTemplateRegistry::default();
    registry
        .register_document("workbench.shell.file", document)
        .unwrap();

    let instance = registry.instantiate("workbench.shell.file").unwrap();
    assert_eq!(instance.root.component.as_deref(), Some("WorkbenchShell"));
    assert_eq!(instance.root.children.len(), 3);
    assert_eq!(
        instance.root.children[0].component.as_deref(),
        Some("UiHostToolbar")
    );
    assert_eq!(
        instance.root.children[1].component.as_deref(),
        Some("HorizontalBox")
    );
    assert_eq!(
        instance.root.children[2].component.as_deref(),
        Some("StatusBar")
    );
    assert_eq!(instance.root.children[1].children.len(), 2);
    assert_eq!(
        instance.root.children[1].children[0].component.as_deref(),
        Some("ActivityRail")
    );
    assert_eq!(
        instance.root.children[1].children[1].component.as_deref(),
        Some("DocumentHost")
    );
}
