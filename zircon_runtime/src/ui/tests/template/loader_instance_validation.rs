use super::*;

#[test]
fn template_loader_parses_component_slots_and_binding_refs_from_toml() {
    let document = UiTemplateLoader::load_toml_str(WORKBENCH_TEMPLATE_TOML).unwrap();

    assert_eq!(document.version, 1);
    assert_eq!(document.root.template.as_deref(), Some("WorkbenchShell"));

    let shell = document.components.get("WorkbenchShell").unwrap();
    assert!(shell.slots["menu_bar"].required);
    assert!(!shell.slots["menu_bar"].multiple);

    let menu_bar = document.components.get("MenuBar").unwrap();
    let toolbar_root = &menu_bar.root;
    assert_eq!(toolbar_root.component.as_deref(), Some("UiHostToolbar"));
    assert_eq!(toolbar_root.children.len(), 2);
    assert_eq!(
        toolbar_root.children[0].bindings[0].id,
        "WorkbenchMenuBar/OpenProject"
    );
    assert_eq!(
        toolbar_root.children[0].bindings[0].event,
        UiEventKind::Click
    );
    assert_eq!(
        toolbar_root.children[0].bindings[0].route.as_deref(),
        Some("MenuAction.OpenProject")
    );
}

#[test]
fn template_instance_expands_composite_slots_and_preserves_stable_bindings() {
    let document = UiTemplateLoader::load_toml_str(WORKBENCH_TEMPLATE_TOML).unwrap();
    UiTemplateValidator::validate_document(&document).unwrap();

    let instance = UiTemplateInstance::from_document(&document).unwrap();

    assert_eq!(instance.root.component.as_deref(), Some("WorkbenchShell"));
    assert_eq!(instance.root.children.len(), 3);
    assert_eq!(
        instance.root.children[0].component.as_deref(),
        Some("UiHostToolbar")
    );
    assert_eq!(
        instance.root.children[1].component.as_deref(),
        Some("ActivityRail")
    );
    assert_eq!(
        instance.root.children[2].control_id.as_deref(),
        Some("DocumentHost")
    );

    let bindings = instance.binding_refs();
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].id, "WorkbenchMenuBar/OpenProject");
    assert_eq!(bindings[0].route.as_deref(), Some("MenuAction.OpenProject"));
    assert_eq!(bindings[1].id, "WorkbenchMenuBar/SaveProject");
    assert_eq!(bindings[1].route.as_deref(), Some("MenuAction.SaveProject"));
}

#[test]
fn template_validator_rejects_missing_required_slots() {
    let document = UiTemplateLoader::load_toml_str(
        r#"
version = 1

[root]
template = "WorkbenchShell"

[components.WorkbenchShell]
slots = { menu_bar = { required = true } }
root = { component = "WorkbenchShell", children = [{ slot = "menu_bar" }] }
"#,
    )
    .unwrap();

    let error = UiTemplateValidator::validate_document(&document).unwrap_err();
    assert_eq!(
        error,
        UiTemplateError::MissingRequiredSlot {
            template_id: "WorkbenchShell".to_string(),
            slot_name: "menu_bar".to_string(),
        }
    );
}
